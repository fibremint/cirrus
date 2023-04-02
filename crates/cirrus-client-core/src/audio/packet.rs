use std::{sync::{Arc, Mutex}, collections::HashMap, fmt::Display};

use anyhow::anyhow;
use cirrus_protobuf::api::AudioDataRes;
use rand::Rng;

type PacketIndex = u32;


pub struct BufChunkInfoNode {
    pub id: u32,
    pub start_idx: u32,
    pub end_idx: u32,

    pub prev_info: Option<Arc<Mutex<BufChunkInfoNode>>>,
    pub next_info: Option<Arc<Mutex<BufChunkInfoNode>>>,
}

impl BufChunkInfoNode {
    pub fn new(
        idx_from: u32, 
        prev_info: Option<Arc<Mutex<BufChunkInfoNode>>>, 
        next_info: Option<Arc<Mutex<BufChunkInfoNode>>>
    ) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();

        Self { 
            id,
            start_idx: idx_from,
            // end_idx: idx_from+1,
            end_idx: idx_from,
            prev_info, 
            next_info 
        }
    }
}

pub struct EncodedBuffer {
    pub content_packets: u32,
    pub frame_buf: HashMap<u32, AudioDataRes>, // packet idx, packet
    pub buf_chunk_info: HashMap<u32, Arc<Mutex<BufChunkInfoNode>>>,
    pub seek_buf_chunk_node_idx: u32,
    first_node_id: PacketIndex,
    pub last_node_id: PacketIndex,
    pub next_packet_idx: PacketIndex,
}

impl EncodedBuffer {
    pub fn new(content_packets: u32) -> Self {
        let mut buf_chunk_info = HashMap::new();

        let bci_node = BufChunkInfoNode::new(0, None, None);
        let bci_node_id = bci_node.id;
        let bci_node = Arc::new(Mutex::new(bci_node));

        buf_chunk_info.insert(bci_node_id.try_into().unwrap(), bci_node);

        Self {
            content_packets,
            frame_buf: Default::default(), 
            buf_chunk_info: buf_chunk_info,
            seek_buf_chunk_node_idx: bci_node_id,
            first_node_id: bci_node_id,
            last_node_id: bci_node_id,
            next_packet_idx: 0,
        }
    }
}

struct CI {
    pub curr: Option<Arc<Mutex<BufChunkInfoNode>>>,
    pub found: Option<Arc<Mutex<BufChunkInfoNode>>>,
    pub search_dir: NodeSearchDirection,
}

impl CI {
    fn new(search_from: Arc<Mutex<BufChunkInfoNode>>, search_dir: NodeSearchDirection) -> Self {
        Self {
            curr: Some(search_from),
            found: None,
            search_dir
        }
    }
}

impl Iterator for CI {
    type Item = Arc<Mutex<BufChunkInfoNode>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_none() {
            return None
        }

        self.found = self.curr.clone();

        let binding = self.curr.as_ref().unwrap();
        let c = binding.lock().unwrap();

        let next_search_node = match self.search_dir {
            NodeSearchDirection::Forward => c.next_info.clone(),
            NodeSearchDirection::Backward => c.prev_info.clone(),
        };

        drop(c);

        self.curr = next_search_node;
        self.found.clone()
    }
}

impl EncodedBuffer {
    fn print_nodes(&self) {
        let bci = self.buf_chunk_info.get(&self.first_node_id).unwrap();
        let mut ci = CI::new(bci.clone(), NodeSearchDirection::Forward);

        while let Some(cur) = ci.next() {
            let c = cur.lock().unwrap();

            println!("({}..{}) - {}", c.start_idx, c.end_idx, c.id);

            if c.prev_info.is_some() {
                let p = c.prev_info.as_ref().unwrap();
                let p = p.lock().unwrap();
                println!("prev: ({}..{})", p.start_idx, p.end_idx);
            }
            if c.next_info.is_some() {
                let n = c.next_info.as_ref().unwrap();
                let n = n.lock().unwrap();

                println!("next: ({}..{})", n.start_idx, n.end_idx);
            }
        }
    
    }

    fn append_new_node_from(&mut self, prev_node: Arc<Mutex<BufChunkInfoNode>>, packet_idx: u32) -> PacketIndex {
        let prev_node_next = prev_node.lock().unwrap().next_info.clone();

        let new_node = BufChunkInfoNode::new(
            packet_idx, 
            Some(prev_node.clone()), 
            prev_node_next
        );

        let new_node_id = new_node.id;
        let next_node = new_node.next_info.clone();

        let new_node = Arc::new(Mutex::new(new_node));

        {
            let mut pn = prev_node.lock().unwrap();
            pn.next_info = Some(Arc::clone(&new_node));
    
            if next_node.is_some() {
                let next_node = next_node.unwrap();
                let mut nn = next_node.lock().unwrap();
                nn.prev_info = Some(Arc::clone(&new_node));
            }
        }

        self.buf_chunk_info.insert(new_node_id, Arc::clone(&new_node));

        let last_chunk = self.get_last_chunk().unwrap();
        self.last_node_id = last_chunk.lock().unwrap().id;

        new_node_id
    }

    pub fn merge_node_from_current(&mut self) {
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let mut bc = bci_node.lock().unwrap();

        if bc.next_info.is_none() {
            return;
        }

        let next_node = bc.next_info.clone().unwrap();
        let nn = next_node.lock().unwrap();

        if bc.end_idx < nn.start_idx {
            return;
        }

        bc.next_info = nn.next_info.clone();
        bc.end_idx = nn.end_idx;

        if let Some(nn_next_node) = &bc.next_info {
            nn_next_node.lock().unwrap().prev_info = Some(Arc::clone(&bci_node));
        }

        self.next_packet_idx = nn.end_idx;
        self.buf_chunk_info.remove(&nn.id);

        drop(nn);
        drop(bc);

        let last_chunk = self.get_last_chunk().unwrap();
        self.last_node_id = last_chunk.lock().unwrap().id;
    }

    pub fn insert(&mut self, audio_data: AudioDataRes) {
        {
            let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
            let mut bc = bci_node.lock().unwrap();

            let pkt_idx = audio_data.packet_idx;

            if bc.next_info.is_some() {
                if audio_data.packet_idx >= bc.next_info.as_ref().unwrap().lock().unwrap().start_idx {
                    println!("current chunk exceeds next chunk");
                }
            }
    
            if let Some(p) = self.frame_buf.insert(audio_data.packet_idx, audio_data) {
                println!("duplicated item insert: prev: {:?}", p.packet_idx);
            }

            bc.end_idx = pkt_idx +1;
            self.next_packet_idx = bc.end_idx;
        }

        self.merge_node_from_current();
    }

    pub fn get_fetch_required_packet_num(&self, fetch_start_idx: u32, duration_sec: Option<f64>) -> u32 {
        let max_avail_fetch_pkt = self.content_packets -1 - fetch_start_idx;

        // let desired_fetch_pkt_num = get_packet_idx_from_sec(duration_sec, 0.02);
        let desired_fetch_pkt_num = 
            if duration_sec.is_some() {
                get_packet_idx_from_sec(duration_sec.unwrap(), 0.02)
            } else {
                std::usize::MAX
            };
            
        let default_val = std::cmp::min(desired_fetch_pkt_num, max_avail_fetch_pkt.try_into().unwrap());

        let bci = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap();
        let nn = bci.lock().unwrap().next_info.clone();

        if nn.is_none() {
            return default_val.try_into().unwrap();
        }

        let nn_start_idx = nn.unwrap().lock().unwrap().start_idx as i32;
        let fetch_start_idx = fetch_start_idx as i32;

        std::cmp::min(default_val.try_into().unwrap(), (nn_start_idx - fetch_start_idx).try_into().unwrap())
    }

    pub fn update_seek_position(
        &mut self, 
        from_packet_idx: u32,
        to_packet_idx: u32, 
    ) {
        let direction = if to_packet_idx as i32 - from_packet_idx as i32 > 0 {
            NodeSearchDirection::Forward
        } else {
            NodeSearchDirection::Backward
        };

        println!("previous seek index: {}", self.seek_buf_chunk_node_idx);
        println!("direction: {:?}", direction);

        // let packet_idx = get_packet_idx_from_sec(position_sec, 0.02) as u32;
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap();                
        
        {
            let bc = bci_node.lock().unwrap();
            if bc.start_idx <= to_packet_idx &&
                bc.end_idx > to_packet_idx {
                    self.seek_buf_chunk_node_idx = bc.id;
                    return;
                }
        }

        let mut ci = CI::new(bci_node.clone(), direction);
        let mut new_node_id: Option<u32> = None;

        while let Some(s) = ci.next() {
            let c = s.lock().unwrap();

            let node_id = c.id;
            let start_idx = c.start_idx;
            let end_idx = c.end_idx;
            let prev_node = c.prev_info.clone();
            let next_node = c.next_info.clone();

            drop(c);

            if start_idx <= to_packet_idx && end_idx > to_packet_idx {
                self.seek_buf_chunk_node_idx = node_id;
                break;
            }

            match direction {
                NodeSearchDirection::Forward => {                    
                    if start_idx > to_packet_idx {
                        let nid = self.append_new_node_from(
                            prev_node.unwrap(),
                            to_packet_idx
                        );

                        new_node_id = Some(nid);
                        break;
                    }
                    
                    if next_node.is_none() {
                        let nid = self.append_new_node_from(
                            s.clone(), 
                            to_packet_idx
                        );
                        
                        new_node_id = Some(nid);
                        break;
                    }
                },
                NodeSearchDirection::Backward => {
                    if end_idx <= to_packet_idx {
                        let nid = self.append_new_node_from(
                            s.clone(), 
                            to_packet_idx
                        );
                        
                        new_node_id = Some(nid);
                        break;                    
                    }
                },
            }
        }

        if let Some(nid) = new_node_id {
            // self.print_nodes();
            self.seek_buf_chunk_node_idx = nid;
        }

        println!("updated seek index: {}", self.seek_buf_chunk_node_idx);

        self.set_buf_reqest_idx(to_packet_idx);
    }

    fn set_buf_reqest_idx(&mut self, packet_idx: u32) {
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let bc = bci_node.lock().unwrap();

        // let request_packet_idx = get_packet_idx_from_sec(position_sec, 0.02) as u32;

        if packet_idx > bc.end_idx || 
            packet_idx < bc.start_idx {
                self.next_packet_idx = packet_idx;
                return;
            }

        self.next_packet_idx = bc.end_idx
    }

    pub fn get_last_chunk(&self) -> Option<Arc<Mutex<BufChunkInfoNode>>> {
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let mut ci = CI::new(bci_node, NodeSearchDirection::Forward);

        let mut last_chunk: Option<Arc<Mutex<BufChunkInfoNode>>> = None;

        while let Some(curr) = ci.next() {
            let c = curr.lock().unwrap();

            if c.next_info.is_none() {
                last_chunk = Some(curr.clone());
            }
        }

        last_chunk
    }

    pub fn get_chunks_num_from_current(&self) -> u32 {
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let ci = CI::new(bci_node, NodeSearchDirection::Forward);

        ci.into_iter().count().try_into().unwrap()
    }

    pub fn is_filled_all_packets(&self) -> bool {
        // TODO: check
        assert!(self.next_packet_idx <= self.content_packets);
        if self.next_packet_idx == 0 {
            return false
        }

        self.next_packet_idx -1 == self.content_packets 
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NodeSearchDirection {
    Forward,
    Backward,
}

pub fn get_packet_idx_from_sec(sec: f64, packet_dur: f64) -> usize {
    (sec / packet_dur).floor() as usize
}



//////////

struct BufferNode {
    id: u32,
    buf_start_idx: Option<u32>,
    buf_end_idx: Option<u32>,

    prev_node_id: Option<u32>,
    next_node_id: Option<u32>,
}

#[derive(Debug)]
enum BufferNodeAddError {
    BeforeStartIdx,
    AfterEndIdx,
}

impl Display for BufferNodeAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeStartIdx => write!(f, "BeforeStartIdx"),
            Self::AfterEndIdx => write!(f, "AfterEndIdx"),
        }
    }
}

impl BufferNode {
    fn new(
        // initial_buf_idx: u32,
        prev_node_id: Option<u32>,
        next_node_id: Option<u32>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();

        Self {
            id,
            // buf_start_idx: initial_buf_idx,
            // buf_end_idx: initial_buf_idx,
            buf_start_idx: Default::default(),
            buf_end_idx: Default::default(),
            prev_node_id,
            next_node_id,
        }
    }

    fn add_packet(
        &mut self,
        packet_idx: u32,
    ) -> anyhow::Result<(), anyhow::Error> {
        if self.buf_start_idx.is_none() {
            self.buf_start_idx = Some(packet_idx);
            self.buf_end_idx = Some(packet_idx);

            return Ok(())
        }

        if self.buf_start_idx.unwrap() > packet_idx {
            return Err(anyhow!(BufferNodeAddError::BeforeStartIdx))
        }

        if self.buf_end_idx.unwrap() +1 != packet_idx {
            return Err(anyhow!(BufferNodeAddError::AfterEndIdx)) 
        }

        // self.buf_end_idx.map(|mut item| item += 1);
        // self.buf_end_idx.as_mut().

        if let Some(ref mut buf_end_idx) = self.buf_end_idx {
            *buf_end_idx = packet_idx;
        }

        Ok(())
    }
}

struct BufferContext {
    buffer_nodes: HashMap<u32, BufferNode>,
    first_node_id: Option<u32>,
    last_node_id: Option<u32>,
    current_node_id: Option<u32>,    
}

impl BufferContext {
    fn new() -> Self {

        Self {
            buffer_nodes: Default::default(),
            first_node_id: Default::default(),
            last_node_id: Default::default(),
            current_node_id: Default::default(),
        }
    }

    fn check_overlaps(
        &self,
    ) -> bool {
        let current_node = self.buffer_nodes.get(
            &self.current_node_id.unwrap()
        ).unwrap();

        if let Some(next_id) = current_node.next_node_id {
            let next_node = self.buffer_nodes.get(
                &next_id
            ).unwrap();

            return current_node.buf_end_idx.unwrap() == next_node.buf_start_idx.unwrap();
        } else {
            return false;
        }
    }

    fn find_current_node(
        &self,
        search_direction: NodeSearchDirection,
        target_packet_idx: u32,
    ) -> (Option<u32>, bool) {
        let mut is_append_node_required = false;
        let mut current_node_id = self.current_node_id;

        while current_node_id.is_some() {
            let current_node = self.buffer_nodes.get(
                &current_node_id.unwrap()
            ).unwrap();

            if current_node.buf_start_idx.unwrap() <= target_packet_idx &&
                current_node.buf_end_idx.unwrap() > target_packet_idx {
                    break;
                }

            match search_direction {
                NodeSearchDirection::Backward => {
                    if current_node.buf_end_idx.unwrap() <= target_packet_idx {
                        is_append_node_required = true;
                        break;
                    }

                    current_node_id = current_node.prev_node_id;
                },
                NodeSearchDirection::Forward => {
                    if current_node.next_node_id.is_none() {
                        is_append_node_required = true;
                        break;
                    }

                    let next_node = self.buffer_nodes.get(
                        &current_node.next_node_id.unwrap()
                    ).unwrap();

                    if next_node.buf_start_idx.unwrap() > target_packet_idx {
                        is_append_node_required = true;
                        break;
                    }

                    current_node_id = current_node.next_node_id;
                },
            }
        }

        (current_node_id, is_append_node_required)
    }

    fn resolve_buffer_node_mismatch(
        &mut self,
        mismatch_err: &BufferNodeAddError,
        packet_idx: u32,
    ) {
        let search_direction = match mismatch_err {
            BufferNodeAddError::BeforeStartIdx => NodeSearchDirection::Backward,
            BufferNodeAddError::AfterEndIdx => NodeSearchDirection::Forward,
            _ => unreachable!()
        };

        let (mut current_node_id, is_append_node_required) = self.find_current_node(
            search_direction,
            packet_idx
        );
        // let mut is_append_node_required = false;
        // let mut current_node_id = self.current_node_id;

        // while current_node_id.is_some() {
        //     let current_node = self.buffer_nodes.get(
        //         &current_node_id.unwrap()
        //     ).unwrap();

        //     if current_node.buf_start_idx.unwrap() <= packet_idx &&
        //         current_node.buf_end_idx.unwrap() > packet_idx {
        //             break;
        //         }

        //     match mismatch_err {
        //         BufferNodeAddError::BeforeStartIdx => {
        //             if current_node.buf_end_idx.unwrap() <= packet_idx {
        //                 is_append_node_required = true;
        //                 break;
        //             }

        //             current_node_id = current_node.prev_node_id;
        //         },
        //         BufferNodeAddError::AfterEndIdx => {
        //             if current_node.next_node_id.is_none() {
        //                 is_append_node_required = true;
        //                 break;
        //             }

        //             let next_node = self.buffer_nodes.get(
        //                 &current_node.next_node_id.unwrap()
        //             ).unwrap();

        //             if next_node.buf_start_idx.unwrap() > packet_idx {
        //                 is_append_node_required = true;
        //                 break;
        //             }

        //             current_node_id = current_node.next_node_id;
        //         },
        //     }
        // }

        if is_append_node_required {
            current_node_id = Some(self.create_node_from(
                current_node_id.unwrap()
            ));
        }

        self.current_node_id = current_node_id;
        
    }

    fn insert(
        &mut self,
        packet_idx: u32
    ) -> u32 {
        if self.current_node_id.is_none() {
            self.set_init_buffer();
        }
        
        {
            let current_node = self.buffer_nodes.get_mut(
                &self.current_node_id.unwrap()
            ).unwrap();
    
            match current_node.add_packet(packet_idx) {
                Ok(_) => (),
                Err(err) => {
                    self.resolve_buffer_node_mismatch(
                        err.downcast_ref::<BufferNodeAddError>().unwrap(),
                        packet_idx
                    );
    
                    let current_node = self.buffer_nodes.get_mut(
                        &self.current_node_id.unwrap()
                    ).unwrap();
    
                    current_node.add_packet(packet_idx).unwrap();    
                }
            }
        }

        if self.check_overlaps() {
            self.merge_node_from_current();
            println!("node merged");
        }

        let current_node = self.buffer_nodes.get(
            &self.current_node_id.unwrap()
        ).unwrap();

        current_node.buf_end_idx.unwrap() +1
    }

    fn create_node_from(
        &mut self,
        node_id: u32,
    ) -> u32 {
        let current_node = self.buffer_nodes.get_mut(
            &node_id
        ).unwrap();

        // let new_node = BufferNode::new(
        //     current_node.prev_node_id,
        //     current_node.next_node_id,
        // );
        
        let new_node = BufferNode::new(
            Some(current_node.id),
            current_node.next_node_id,
        );

        current_node.next_node_id = Some(new_node.id);

        if new_node.next_node_id.is_some() {
            let next2_node_id = new_node.next_node_id.unwrap();
            let mut next2_node = self.buffer_nodes.get_mut(
                &next2_node_id
            ).unwrap();

            next2_node.prev_node_id = Some(new_node.id);
        }

        // if current_node.next_node_id.is_some() {
        //     let next_node_id = current_node.next_node_id.unwrap();
        //     let mut next_node = self.buffer_nodes.get_mut(
        //         &next_node_id
        //     ).unwrap();

        //     next_node.prev_node_id = Some(new_node.id);
        // }

        let new_node_id = new_node.id;
        self.buffer_nodes.insert(new_node_id, new_node);

        new_node_id
    }

    fn merge_node_from_current(
        &mut self,
    ) {
        let (
            current_node_id,
            next_node_id,
            next_buf_end_idx,
            next2_node_id,
        ) = {
            let current_node = self.buffer_nodes.get(
                &self.current_node_id.unwrap()
            ).unwrap();

            let nid = current_node.next_node_id.unwrap();
            let next_node = self.buffer_nodes.get(
                &nid
            ).unwrap();

            (current_node.id, nid, next_node.buf_end_idx, next_node.next_node_id)
        };

        let current_node = self.buffer_nodes.get_mut(
            &current_node_id
        ).unwrap();

        current_node.buf_end_idx = next_buf_end_idx;
        current_node.next_node_id = next2_node_id;

        if next2_node_id.is_some() {
            let next2_node = self.buffer_nodes.get_mut(
                &next2_node_id.unwrap()
            ).unwrap();

            next2_node.prev_node_id = Some(current_node_id);
        }

        // if let Some(next_next_node) = self.buffer_nodes.get_mut(
        //     &next_next_node_id.unwrap()
        // ) {
        //     next_next_node.prev_node_id = Some(current_node_id);
        // }

        // if let Some(next_next_node) = self.buffer_nodes.get_mut(
        //     &next_next_node_id.unwrap()
        // ) {
        //     next_next_node.prev_node_id = Some(current_node_id);
        // }

        if self.last_node_id.unwrap() == next_node_id {
            self.last_node_id = Some(current_node_id);
        }

        self.buffer_nodes.remove(&next_node_id);
    }

    fn set_init_buffer(
        &mut self,
        // packet_idx: u32
    ) {
        let buffer_node = BufferNode::new(
            // packet_idx,
            None,
            None
        );

        self.first_node_id = Some(buffer_node.id);
        self.last_node_id = Some(buffer_node.id);
        self.current_node_id = Some(buffer_node.id);

        self.buffer_nodes.insert(buffer_node.id, buffer_node);
    }

    fn clear(
        &mut self,
    ) {
        self.first_node_id = None;
        self.last_node_id = None;
        self.current_node_id = None;
    }
}

enum FetchStatus {
    Init,
    Pending,
    Fetched,
    Exists,
    Error
}

impl From<usize> for FetchStatus {
    fn from(value: usize) -> Self {
        use self::FetchStatus::*;
        match value {
            0 => Init,
            1 => Pending,
            2 => Fetched,
            3 => Exists,
            41 => Error,
            _ => unreachable!(),
        }
    }
}

struct PacketFetchStatus {
    packet_idx: Option<u32>,
    status: usize,
}

impl Default for PacketFetchStatus {
    fn default() -> Self {
        Self { 
            packet_idx: Default::default(),
            status: Default::default() 
        }
    }
}

pub struct PacketBuffer {
    data: HashMap<u32, AudioDataRes>,
    ctx: BufferContext,
    
    max_packet_idx: u32,
    // packet_fetch_status: PacketFetchStatus,
    // next_fetch_idx: Option<u32>
    prev_fetched_idx: Option<u32>,
}

impl PacketBuffer {
    pub fn new(
        content_packets: u32
    ) -> Self {

        Self {
            data: Default::default(),
            ctx: BufferContext::new(),
            max_packet_idx: content_packets -1,
            // packet_fetch_status: Default::default(),
            // next_fetch_idx: Default::default(),
            prev_fetched_idx: Default::default(),
        }
    }

    pub fn insert(
        &mut self,
        audio_data: AudioDataRes
    ) {
        let next_fetch_idx = self.ctx.insert(audio_data.packet_idx);
        
        self.prev_fetched_idx = Some(audio_data.packet_idx);

        // if next_fetch_idx <= self.max_packet_idx {
        //     self.next_fetch_idx = Some(next_fetch_idx)
        // }

        if let Some(d) = self.data.insert(
            audio_data.packet_idx,
            audio_data
        ) {
            println!("WARN: duplicated item inserted, idx: {}", d.packet_idx);
        }
        
        // if let Some(ref mut next_packet_idx) = self.next_fetch_idx {
            
        // }
        // if next_ {
        //     self.next_fetch_idx = Some(audio_data.packet_idx +1);
        // }
    }

    pub fn get_data(
        &self,
        packet_idx: u32
    ) -> Option<&AudioDataRes> {
        self.data.get(&packet_idx)
    }

    // pub fn get_fetch_start_idx(
    //     &self
    // ) -> Option<u32> {
    //     if self.ctx.current_node_id.is_none() {
    //         return None
    //     }

    //     let current_node = self.ctx.buffer_nodes.get(
    //         &self.ctx.current_node_id.unwrap()
    //     ).unwrap();

    //     current_node.buf_end_idx
    // }

    pub fn get_fetch_start_packet_idx(
        &self,
        playback_position: u32,
    ) -> u32 {
        if self.ctx.current_node_id.is_none() {
            return playback_position;
        }

        // if self.prev_fetched_idx.is_none() {
        //     return playback_position;
        // }

        let current_node = self.ctx.buffer_nodes.get(
            &self.ctx.current_node_id.unwrap()
        ).unwrap();

        // if current_node.buf_end_idx.unwrap() == self.prev_fetched_idx.unwrap() {
        if self.prev_fetched_idx.is_some() &&
            current_node.buf_end_idx.unwrap() == self.prev_fetched_idx.unwrap() {
            
            return self.prev_fetched_idx.unwrap() +1;
        }

        // if current_node.buf_start_idx.unwrap() <= playback_position && 
        //     playback_position <= current_node.buf_end_idx.unwrap() {
            
        //     return current_node.buf_end_idx.unwrap() +1
        // }

        let search_direction = 
            if current_node.buf_start_idx.unwrap() > playback_position
                { NodeSearchDirection::Backward }
            // TODO: check this
            // else if current_node.buf_end_idx.unwrap() < playback_position
            else
                { NodeSearchDirection::Forward };

        let (found_node_id, is_append_node_required) = self.ctx.find_current_node(
            search_direction,
            playback_position
        );

        if is_append_node_required {
            return playback_position;
        }

        let found_node = self.ctx.buffer_nodes.get(
            &found_node_id.unwrap()
        ).unwrap();

        // self.ctx.current_node_id = found_node_id;
        found_node.buf_end_idx.unwrap() +1

        // if current_node.buf_end_idx.unwrap() < playback_position ||
        //     current_node.buf_start_idx.unwrap() > playback_position {

        //     return playback_position;
        // }

        // current_node.buf_end_idx.unwrap() +1
    }

    // pub fn set_next_fetch_idx(
    //     &mut self,
    //     idx: u32,
    // ) {
    //     self.next_fetch_idx = Some(idx);
    // }

    pub fn clear_previous_fetch_idx(
        &mut self
    ) {
        self.prev_fetched_idx = None;
    }

    pub fn get_remain_buffer(
        &self,
        position: u32,
    ) -> u32 {
        if self.ctx.current_node_id.is_none() {
            return 0;
        }

        let current_node = self.ctx.buffer_nodes.get(
            &self.ctx.current_node_id.unwrap()
        ).unwrap();

        // let relative_position = current_node.buf_start_idx.unwrap() as i32 - position as i32;
        let relative_position = position as i32 - current_node.buf_start_idx.unwrap() as i32;
        if relative_position < 0 {
            return 0;
        }

        let buffer_len = current_node.buf_end_idx.unwrap() as i32 - current_node.buf_start_idx.unwrap() as i32;
        
        std::cmp::max(0, buffer_len - relative_position) as u32
        // std::cmp::max(0, relative_position - buffer_len) as u32

        // std::cmp::max(current_node.buf_end_idx.unwrap() as i32 - position as i32, 0) as u32

        // (position - current_node.buf_start_idx)
        // (current_node.buf_end_idx - current_node.buf_start_idx)
    }

    pub fn get_fetch_required_packet_num(
        &self,
        desired_fetch_packets: u32,
    ) -> u32 {
        // let mut fetch_packets = desired_fetch_packets;

        if self.ctx.current_node_id.is_none() {
            return std::cmp::min(desired_fetch_packets, self.max_packet_idx);
        }

        let current_node = self.ctx.buffer_nodes.get(
            &self.ctx.current_node_id.unwrap()
        ).unwrap();

        let max_packet_idx = {
            match current_node.next_node_id {
                Some(next_node_id) => {
                    let next_node = self.ctx.buffer_nodes.get(
                        &next_node_id
                    ).unwrap();

                    next_node.buf_start_idx.unwrap()
                },
                None => self.max_packet_idx,
            }
        };

        // if let Some(next_node_id) = current_node.next_node_id {
        //     let next_node = self.ctx.buffer_nodes.get(
        //         &next_node_id
        //     ).unwrap();

        //     fetch_packets = std::cmp::min(
        //         fetch_packets,
        //         next_node.buf_start_idx.unwrap() - current_node.buf_end_idx.unwrap()
        //     );
        // }

        std::cmp::min(
            desired_fetch_packets,
            max_packet_idx - current_node.buf_end_idx.unwrap(),
        )
    }

    pub fn is_filled_all(
        &self
    ) -> bool {
        // TODO: edit check condition
        self.ctx.buffer_nodes.len() == 1
    }

    // pub fn fetch(
    //     &mut self,
    //     fetch_sec: u32,
    // ) {
    //     todo!()
    // }

    fn clear(
        &mut self,
    ) {
        self.data.clear();
        self.ctx.clear();
    }
}

