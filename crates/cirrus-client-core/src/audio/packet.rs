use std::{collections::HashMap, fmt::Display};

use anyhow::anyhow;
use cirrus_protobuf::api::AudioDataRes;
use rand::Rng;
use itertools::Itertools;

#[derive(Clone, Copy, Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

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
        prev_node_id: Option<u32>,
        next_node_id: Option<u32>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();

        Self {
            id,
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

            return current_node.buf_end_idx.unwrap() +1 == next_node.buf_start_idx.unwrap();
        } else {
            return false;
        }
    }

    fn find_current_node(
        &self,
        search_direction: SearchDirection,
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
                SearchDirection::Backward => {
                    if current_node.buf_end_idx.unwrap() <= target_packet_idx {
                        is_append_node_required = true;
                        break;
                    }

                    current_node_id = current_node.prev_node_id;
                },
                SearchDirection::Forward => {
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
            BufferNodeAddError::BeforeStartIdx => SearchDirection::Backward,
            BufferNodeAddError::AfterEndIdx => SearchDirection::Forward,
        };

        let (mut current_node_id, is_append_node_required) = self.find_current_node(
            search_direction,
            packet_idx
        );

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
    ) -> Result<(), anyhow::Error> {
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
                    let before_current_node_id = current_node.id;

                    self.resolve_buffer_node_mismatch(
                        err.downcast_ref::<BufferNodeAddError>().unwrap(),
                        packet_idx
                    );
    
                    let resolved_current_node = self.buffer_nodes.get_mut(
                        &self.current_node_id.unwrap()
                    ).unwrap();
    
                    if let Err(_) = resolved_current_node.add_packet(packet_idx) {
                        eprintln!("before current id: {}", before_current_node_id);
                        eprintln!("resolved current id: {}", resolved_current_node.id);

                        for (k, v) in self.buffer_nodes
                            .iter()
                            .sorted_by(|a, b| Ord::cmp(&a.1.buf_start_idx, &b.1.buf_start_idx)) {

                                eprintln!("node: {} {}..{}", k, v.buf_start_idx.unwrap(), v.buf_end_idx.unwrap());
                            }

                        return Err(anyhow!("failed to resolve node mismatch"));
                    }
                }
            }
        }

        if self.check_overlaps() {
            self.merge_node_from_current();
        }

        Ok(())
    }

    fn create_node_from(
        &mut self,
        node_id: u32,
    ) -> u32 {
        let current_node = self.buffer_nodes.get_mut(
            &node_id
        ).unwrap();

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

        let current_node_buf_prev_end_idx = current_node.buf_end_idx.unwrap();

        current_node.buf_end_idx = next_buf_end_idx;
        current_node.next_node_id = next2_node_id;

        if next2_node_id.is_some() {
            let next2_node = self.buffer_nodes.get_mut(
                &next2_node_id.unwrap()
            ).unwrap();

            next2_node.prev_node_id = Some(current_node_id);
        }

        if self.last_node_id.unwrap() == next_node_id {
            self.last_node_id = Some(current_node_id);
        }

        let next_node = self.buffer_nodes.get(
            &next_node_id
        ).unwrap();

        let current_node = self.buffer_nodes.get(
            &current_node_id
        ).unwrap();

        println!("node merged: ({}..{}), ({}..{}) -> ({}..{}) ", 
            current_node.buf_start_idx.unwrap(),
            current_node_buf_prev_end_idx,

            &next_node.buf_start_idx.unwrap(),
            &next_node.buf_end_idx.unwrap(),

            current_node.buf_start_idx.unwrap(),
            current_node.buf_end_idx.unwrap(),
        );

        self.buffer_nodes.remove(&next_node_id);
    }

    fn set_init_buffer(&mut self) {
        let buffer_node = BufferNode::new(
            None,
            None
        );

        self.first_node_id = Some(buffer_node.id);
        self.last_node_id = Some(buffer_node.id);
        self.current_node_id = Some(buffer_node.id);

        self.buffer_nodes.insert(buffer_node.id, buffer_node);
    }

    // fn clear(&mut self) {
    //     self.first_node_id = None;
    //     self.last_node_id = None;
    //     self.current_node_id = None;
    // }
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
            4 => Error,
            _ => unreachable!(),
        }
    }
}

pub struct PacketBuffer {
    data: HashMap<u32, AudioDataRes>,
    ctx: BufferContext,
    
    max_packet_idx: u32,
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
            prev_fetched_idx: Default::default(),
        }
    }

    pub fn insert(
        &mut self,
        audio_data: AudioDataRes
    ) -> Result<(), anyhow::Error> {
        self.ctx.insert(audio_data.packet_idx)?;
        
        self.prev_fetched_idx = Some(audio_data.packet_idx);

        if let Some(d) = self.data.insert(
            audio_data.packet_idx,
            audio_data
        ) {
            eprintln!("WARN: duplicated item inserted, idx: {}", d.packet_idx);
        }

        Ok(())
    }

    pub fn get_data(
        &self,
        packet_idx: u32
    ) -> Option<&AudioDataRes> {
        self.data.get(&packet_idx)
    }

    pub fn fetch_buffer_guidance(
        &mut self,
        default_start_idx: u32,
        desired_fetch_packets: u32,
    ) -> (u32, u32) {
        
        if self.ctx.current_node_id.is_none() {
            return (
                default_start_idx,
                self.calc_avail_fetch_packets(
                    desired_fetch_packets,
                    None,
                )
            );
        }

        let current_node = self.ctx.buffer_nodes.get(
            &self.ctx.current_node_id.unwrap()
        ).unwrap();

        if current_node.buf_start_idx.unwrap() <= default_start_idx &&
            current_node.buf_end_idx.unwrap() >= default_start_idx {

                return (
                    current_node.buf_end_idx.unwrap() +1,
                    self.calc_avail_fetch_packets(
                        desired_fetch_packets,
                        None,
                    )
                )
            }

        let search_direction = 
            if current_node.buf_start_idx.unwrap() > default_start_idx
                { SearchDirection::Backward }
            else
                { SearchDirection::Forward };

        let (found_node_id, is_append_node_required) = self.ctx.find_current_node(
            search_direction,
            default_start_idx
        );

        self.ctx.current_node_id = found_node_id;

        if is_append_node_required {
            return (
                default_start_idx,
                self.calc_avail_fetch_packets(
                    desired_fetch_packets,
                    Some(default_start_idx)
                )
            );
        }

        let found_node = self.ctx.buffer_nodes.get(
            &found_node_id.unwrap()
        ).unwrap();

        (
            found_node.buf_end_idx.unwrap() +1,
            self.calc_avail_fetch_packets(
                desired_fetch_packets,
                // false,
                None
            )
        )
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

        let relative_position = position as i32 - current_node.buf_start_idx.unwrap() as i32;
        if relative_position < 0 {
            return 0;
        }

        let buffer_len = current_node.buf_end_idx.unwrap() as i32 - current_node.buf_start_idx.unwrap() as i32;
        
        std::cmp::max(0, buffer_len - relative_position) as u32
    }

    fn calc_avail_fetch_packets(
        &self,
        desired_fetch_packets: u32,
        new_node_init_idx: Option<u32>,
    ) -> u32 {

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

        let fetch_start_idx = {
            if let Some(new_node_idx) = new_node_init_idx {
                new_node_idx
            } else {
                current_node.buf_end_idx.unwrap() +1
            }
        };

        let avail_fetch_num = max_packet_idx - fetch_start_idx;

        println!("current_node: {}, ({}..{})", current_node.id, current_node.buf_start_idx.unwrap(), current_node.buf_end_idx.unwrap());

        println!("max packet idx: {}", max_packet_idx);
        println!("delta: {}", avail_fetch_num);

        std::cmp::min(
            desired_fetch_packets,
            avail_fetch_num
        )
    }

    pub fn is_filled_all(
        &self
    ) -> bool {
        // TODO: edit check condition
        self.ctx.buffer_nodes.len() == 1
    }

    // fn clear(
    //     &mut self,
    // ) {
    //     self.data.clear();
    //     self.ctx.clear();
    // }
}

