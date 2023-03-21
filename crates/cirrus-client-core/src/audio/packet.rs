use std::{sync::{Arc, Mutex}, collections::HashMap};

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
        let max_avail_fetch_pkt = self.content_packets - fetch_start_idx;

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

    pub fn update_seek_position(&mut self, position_sec: f64, direction: NodeSearchDirection) {
        println!("previous seek index: {}", self.seek_buf_chunk_node_idx);
        println!("direction: {:?}", direction);

        let packet_idx = get_packet_idx_from_sec(position_sec, 0.02) as u32;
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap();                
        
        {
            let bc = bci_node.lock().unwrap();
            if bc.start_idx <= packet_idx &&
                bc.end_idx > packet_idx {
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

            if start_idx <= packet_idx && end_idx > packet_idx {
                self.seek_buf_chunk_node_idx = node_id;
                break;
            }

            match direction {
                NodeSearchDirection::Forward => {                    
                    if start_idx > packet_idx {
                        let nid = self.append_new_node_from(
                            prev_node.unwrap(),
                            packet_idx
                        );

                        new_node_id = Some(nid);
                        break;
                    }
                    
                    if next_node.is_none() {
                        let nid = self.append_new_node_from(
                            s.clone(), 
                            packet_idx
                        );
                        
                        new_node_id = Some(nid);
                        break;
                    }
                },
                NodeSearchDirection::Backward => {
                    if end_idx <= packet_idx {
                        let nid = self.append_new_node_from(
                            s.clone(), 
                            packet_idx
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

        self.set_buf_reqest_idx(position_sec);
    }

    fn set_buf_reqest_idx(&mut self, position_sec: f64) {
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let bc = bci_node.lock().unwrap();

        let request_packet_idx = get_packet_idx_from_sec(position_sec, 0.02) as u32;

        if request_packet_idx > bc.end_idx || 
            request_packet_idx < bc.start_idx {
                self.next_packet_idx = request_packet_idx;
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
        self.next_packet_idx == self.content_packets 
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
