use std::{sync::{Arc, Mutex}, collections::HashMap, default};

use cirrus_protobuf::api::AudioDataRes;
use opus::packet;
use rand::Rng;


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
            end_idx: idx_from+1,
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
    first_node_id: u32,
    pub next_packet_idx: u32,
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

        // self.curr = Some(next_search_node.unwrap());
        self.curr = next_search_node;

        // if next_search_node.is_none() {
        //     return None;
        // }

        self.curr.clone()
    }
}

impl EncodedBuffer {
    fn find_fit_chunk(
        &self, 
        search_from: &Arc<Mutex<BufChunkInfoNode>>, 
        packet_idx: u32, 
        search_direction: NodeSearchDirection
    ) -> Option<Arc<Mutex<BufChunkInfoNode>>> {
        // let mut curr = search_from.to_owned();
        let mut found_node: Option<Arc<Mutex<BufChunkInfoNode>>> = None;

        let mut ci = CI::new(search_from.clone(), search_direction);
        while let Some(s) = ci.next() {
            let c = s.lock().unwrap();
            if packet_idx >= c.start_idx &&
                packet_idx < c.end_idx {
                    found_node = Some(s.clone());
                    break;
                }
        }

        // loop {
        //     let c = curr.lock().unwrap();

        //     if packet_idx >= c.start_idx &&
        //         packet_idx < c.end_idx {
        //             found_node = Some(curr.clone());
        //             break;
        //         }

        //     // if packet_idx >= c.start_idx &&
        //     //     packet_idx < c.end_idx {
        //     //         found_node = Some(curr.clone());
        //     //         break;
        //     //     }

        //     let next_search_node = match search_direction {
        //         NodeSearchDirection::Forward => c.next_info.clone(),
        //         NodeSearchDirection::Backward => c.prev_info.clone(),
        //     };

        //     if next_search_node.is_none() {
        //         break;
        //     }

        //     drop(c);

        //     curr = next_search_node.unwrap();
        // }

        found_node
    }

    fn update_node(&mut self, audio_data: &AudioDataRes) {
        {
            let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();
            let (bc_start_idx, bc_end_idx) = {
                let bc = bci_node.as_ref().lock().unwrap();
                (bc.start_idx, bc.end_idx)
            };

            if audio_data.packet_idx > bc_end_idx {
                let nn_id = match self.find_fit_chunk(
                    &bci_node, 
                    audio_data.packet_idx, 
                    NodeSearchDirection::Forward
                ) {
                    Some(next_node) => next_node.lock().unwrap().id,
                    None => {
                        let next_node = BufChunkInfoNode::new(
                                audio_data.packet_idx, 
                                Some(bci_node.clone()), 
                                bci_node.lock().unwrap().next_info.clone()
                            );

                        let next_node_id = next_node.id;
                        let nnext_node = next_node.next_info.clone();
                        let next_node = Arc::new(Mutex::new(next_node));

                        if nnext_node.is_some() {
                            nnext_node.unwrap().lock().unwrap().prev_info = Some(Arc::clone(&next_node));
                        }

                        // let next_node_id = next_node.id;
                        // let prev_info = next_node.prev_info.unwrap().clone();
                        // let next_node = Arc::new(Mutex::new(next_node));
                        bci_node.lock().unwrap().next_info = Some(Arc::clone(&next_node));

                        // next_node.prev_info.unwrap().lock().unwrap().next_info = Some(Arc::new(Mutex::new(next_node)));

                        self.buf_chunk_info.insert(next_node_id, Arc::clone(&next_node));

                        next_node_id
                    },
                };

                self.seek_buf_chunk_node_idx = nn_id;
            }

            if audio_data.packet_idx < bc_start_idx {
                let pn_id = match self.find_fit_chunk(
                    &bci_node, 
                    audio_data.packet_idx, 
                    NodeSearchDirection::Backward
                ) {
                    Some(prev_node) => prev_node.lock().unwrap().id,
                    None => {
                        let prev_node = BufChunkInfoNode::new(
                            audio_data.packet_idx, 
                            bci_node.lock().unwrap().prev_info.clone(), 
                            Some(bci_node.clone())
                        );
    
                        let prev_node_id = prev_node.id;
                        let pprev_node = prev_node.prev_info.clone();
                        let prev_node = Arc::new(Mutex::new(prev_node));

                        if pprev_node.is_some() {
                            pprev_node.unwrap().lock().unwrap().next_info = Some(Arc::clone(&prev_node));
                        }

                        // let mut bc = bci_node.lock().unwrap();
                        // bc.prev_info = Some(Arc::clone(&prev_node));

                        bci_node.lock().unwrap().prev_info = Some(Arc::clone(&prev_node));
                        self.buf_chunk_info.insert(prev_node_id, Arc::clone(&prev_node));

                        // self.buf_chunk_info.insert(prev_node_id, Arc::new(Mutex::new(prev_node)));

                        prev_node_id
                    },                
                };

                self.seek_buf_chunk_node_idx = pn_id;
            }
        }

        // Merge chunk

        // let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        // let mut bc = bci_node.lock().unwrap();

        // if bc.next_info.is_none() {
        //     return;
        // }

        // let next_node = bc.next_info.clone().unwrap();
        // let nn = next_node.lock().unwrap();

        // if bc.

        // // if audio_data.packet_idx < nn.start_idx {
        // //     return;
        // // }

        // bc.next_info = nn.next_info.clone();
        // bc.end_idx = nn.end_idx;
        // let nn_id = nn.id;

        // self.buf_chunk_info.remove(&nn_id);
    }

    fn merge_node_from_current(&mut self) {
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

        // check integrity 
        for bc_idx in bc.start_idx..bc.end_idx {
            if let None = self.frame_buf.get(&bc_idx) {
                println!("pre merge: bc detected none at idx: {}", bc_idx);
            }
        }

        // check integrity 
        for nn_idx in nn.start_idx..nn.end_idx {
            if let None = self.frame_buf.get(&nn_idx) {
                println!("pre merge: nn detected none at idx: {}", nn_idx);
            }
        }

        println!("merge info: bc: ({}..{}), nn: ({}..{})", bc.start_idx, bc.end_idx, nn.start_idx, nn.end_idx);

        bc.next_info = nn.next_info.clone();
        bc.end_idx = nn.end_idx;

        if let Some(nn_next_node) = &bc.next_info {
            nn_next_node.lock().unwrap().prev_info = Some(Arc::clone(&bci_node));
        }

        // check integrity 
        for bc_idx in bc.start_idx..bc.end_idx {
            if let None = self.frame_buf.get(&bc_idx) {
                println!("post merge: detected none at idx: {}", bc_idx);
            }
        }

        self.next_packet_idx = nn.end_idx;

        // if let Some(nn_next_node) = &nn.next_info {
        //     nn_next_node.lock().unwrap().prev_info = Some(Arc::clone(&bci_node));
        // }

        self.buf_chunk_info.remove(&nn.id);
    }

    pub fn push(&mut self, audio_data: AudioDataRes) {
        self.update_node(&audio_data);

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

            // check integrity 
            for bc_idx in bc.start_idx..bc.end_idx {
                if let None = self.frame_buf.get(&bc_idx) {
                    println!("push: detected none at idx: {}", bc_idx);
                }
            }

            // bc.end_idx += 1;
            bc.end_idx = pkt_idx +1;
            self.next_packet_idx = bc.end_idx;
        }

        self.merge_node_from_current();
    }

    pub fn get_fetch_required_packet_num(&self, fetch_start_idx: u32, duration_sec: f64) -> u32 {
        // let first_node = self.buf_chunk_info.get(&self.first_node_id).unwrap();
        // let t = self.find_fit_chunk(&first_node, audio_data, search_direction)
        
        // let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        // let bc = bci_node.lock().unwrap();

        // let unf_pckts = match &bc.next_info {
        //     Some(nn) => nn.lock().unwrap().start_idx - fetch_start_idx,
        //     None => self.content_packets - fetch_start_idx,
        // };

        // unf_pckts.try_into().unwrap()

        // let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();
        // let bc = bci_node.lock().unwrap();

        // match &bc.next_info {
        //     Some(nn) => todo!(),
        //     None => ,
        // }

        //////
        /// node.lock().unwrap().start_idx - fetch_start_idx
        /// ///
        ///
        /// 
        /// 
        // let default_val = self.content_packets - fetch_start_idx;
        let max_avail_fetch_pkt = self.content_packets - fetch_start_idx;
        let desired_fetch_pkt_num = get_packet_idx_from_sec(duration_sec, 0.06);
        let default_val = std::cmp::min(desired_fetch_pkt_num, max_avail_fetch_pkt.try_into().unwrap());

        // let default_val = get_packet_idx_from_sec(duration_sec, 0.06) as u32;
        // let first_node = self.buf_chunk_info.get(&self.first_node_id).unwrap();
        let bci = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap();
        let nn = bci.lock().unwrap().next_info.clone();

        if nn.is_none() {
            return default_val.try_into().unwrap();
        }

        let nn_start_idx = nn.unwrap().lock().unwrap().start_idx as i32;
        let fetch_start_idx = fetch_start_idx as i32;

        if nn_start_idx - fetch_start_idx < 0 {
            println!("fix me!");
        }
        
        // std::cmp::min(default_val.try_into().unwrap(), nn_start_idx - fetch_start_idx + 1)
        std::cmp::min(default_val.try_into().unwrap(), (nn_start_idx - fetch_start_idx).try_into().unwrap())
        // let r = {
        //     match &fc.unwrap().lock().unwrap().next_info {
        //         Some(nn) => nn.lock().unwrap().start_idx - fetch_start_idx,
        //         None => default_val,
        //     }    
        // };

        // r

        // let fc = self.find_fit_chunk(
        //     &first_node, 
        //     fetch_start_idx, 
        //     NodeSearchDirection::Forward
        // );

        // drop(first_node);

        // if fc.is_none() {
        //     return default_val;
        // }

        // let r = {
        //     match &fc.unwrap().lock().unwrap().next_info {
        //         Some(nn) => nn.lock().unwrap().start_idx - fetch_start_idx,
        //         None => default_val,
        //     }    
        // };

        // r

        // match self.find_fit_chunk(
        //     &first_node, 
        //     fetch_start_idx, 
        //     NodeSearchDirection::Forward
        // ) {
        //     Some(node) => {
        //         if node.lock().unwrap().next_info.is_some() {

        //         }
        //     },
        //     None => self.content_packets - fetch_start_idx,
        // }
    }

    pub fn update_seek_position(&mut self, position_sec: f64, direction: NodeSearchDirection) {
        let packet_idx = get_packet_idx_from_sec(position_sec, 0.06) as u32;

        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap();                
        
        {
            let bc = bci_node.lock().unwrap();
            if bc.start_idx <= packet_idx &&
                bc.end_idx > packet_idx {
                    return;
                }
        }

        // drop(bc);

        let mut next_node: Option<Arc<Mutex<BufChunkInfoNode>>> = None;
        // let next_node: Arc<Mutex<BufChunkInfoNode>> = None;

        let mut ci = CI::new(bci_node.clone(), direction);
        // let mut i_delta: i32 = 0;
        let mut i_delta = packet_idx as i32 - bci_node.lock().unwrap().start_idx as i32;

        while let Some(s) = ci.next() {
            let c = s.lock().unwrap();

            match direction {
                NodeSearchDirection::Forward => {
                    // if c.start_idx < packet_idx ||
                    //     c.end_idx > packet_idx {
                    //         next_node = Some(s.clone());
                    //         break;
                    //     }
                    if c.next_info.is_none() {
                        next_node = Some(s.clone());
                        break;
                    }

                    if c.start_idx > packet_idx &&
                        i_delta > 0 {
                            next_node = c.prev_info.clone();
                            break;
                        }
                    
                    i_delta = packet_idx as i32 - c.end_idx as i32;
                },
                NodeSearchDirection::Backward => {
                    // if c.start_idx < packet_idx ||
                    //     c.end_idx > packet_idx {
                    //         next_node = Some(s.clone());
                    //         break;
                    //     }
                    // if c.end_idx < packet_idx {
                    //     next_node = Some(s.clone());
                    //     break;
                    // }
                    if c.start_idx > packet_idx {
                        continue;
                    }

                    next_node = Some(s.clone());
                    break;

                    // if c.end_idx -1 < packet_idx || c.start_idx >= packet_idx {
                    //     next_node = Some(s.clone());
                    //     break;
                    // }
                },
            }
        }

        if next_node.is_some() {
            self.seek_buf_chunk_node_idx = next_node.unwrap().lock().unwrap().id;
        }

        self.set_buf_reqest_idx(position_sec);

        // let next_seek_chunk = self.find_fit_chunk(
        //     &bci_node, 
        //     packet_idx.try_into().unwrap(), 
        //     direction
        // );

        // match direction {
        //     NodeSearchDirection::Forward => {

        //     },
        //     NodeSearchDirection::Backward => {

        //     },
        // }

        // self.seek_buf_chunk_node_idx = next_seek_chunk.lock().unwrap().id;

        // match direction {
        //     NodeSearchDirection::Forward => {

        //     },
        //     NodeSearchDirection::Backward => {

        //     },
        // }

        // let first_node = self.buf_chunk_info.get(&self.first_node_id).unwrap();        
        // if let Some(fit_chunk) = self.find_fit_chunk(
        //     first_node, 
        //     packet_idx.try_into().unwrap(), 
        //     NodeSearchDirection::Forward
        // ) {
        //     self.seek_buf_chunk_node_idx = fit_chunk.lock().unwrap().id;
        // }
    }

    fn set_buf_reqest_idx(&mut self, position_sec: f64) {
        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let bc = bci_node.lock().unwrap();

        let request_packet_idx = get_packet_idx_from_sec(position_sec, 0.06) as u32;

        // if request_packet_idx > bc.end_idx {
        //     request_packet_idx
        // } else {
        //     bc.end_idx
        // }

        if request_packet_idx > bc.end_idx || 
            request_packet_idx < bc.start_idx {
                self.next_packet_idx = request_packet_idx;
                return;
            }

        self.next_packet_idx = bc.end_idx
    }
}

#[derive(Clone, Copy)]
pub enum NodeSearchDirection {
    Forward,
    Backward,
}

pub fn get_packet_idx_from_sec(sec: f64, packet_dur: f64) -> usize {
    (sec / packet_dur).floor() as usize
}
