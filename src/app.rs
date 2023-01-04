use std::collections::VecDeque;

use crate::Packet;
use crate::filtable::FiltableList;

#[derive(Debug)]
pub struct App {
    list: FiltableList<Packet>,
    select: Option<usize>,
    view: Option<usize>,
    input_mode: InputMode,
    running: bool,
}

impl std::fmt::Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "filter:{}, select:{:?}, view:{:?}, input_mode:{:?}, running:{}",
            self.get_filter(), self.select, self.view, self.input_mode, self.running
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            list: FiltableList::<Packet>::new(),
            select: None,
            view: None,
            input_mode: InputMode::List,
            running: true,
        }
    }
}

impl App {
    pub fn is_running(&self) -> bool {
        self.running
    }
    pub fn exit(&mut self) {
        self.running = false
    }
    pub fn get_view_text(&self) -> String {
        match self.view {
            Some(x) => self.list.get_item(x).text().join("\n"),
            None => "".to_string(),
        }
    }
    pub fn get_filter(&self) -> String {
        self.list.get_filter()
    }

    pub fn get_view_list(&self, height: u16, offset: &mut usize) -> (Vec<String>, Option<usize>) {
        let (items, select) = self.get_view_list_index(height, offset);
        let ans = items.iter().map(|x| self.list.get_item(*x).line()).collect();
        (ans, select)
    }

    pub fn add_filter_str(&mut self, st: &str) {
        self.list.add_filter_str(st);
    }

    pub fn delete_filter_char(&mut self) {
        self.list.delete_filter_char();
    }

    fn get_view_list_index(&self, height: u16, offset: &mut usize) -> (Vec<usize>, Option<usize>) {
        if self.list.is_filterd_empty() {
            return (vec![], None);
        }
        let mut ans = VecDeque::<usize>::new();
        if self.select.is_none() {
            let mut count = self.list.filterd_last().unwrap();
            while height as usize > ans.len() {
                ans.push_front(count);
                if let Some(x) = self.list.previous(count) {
                    count = x;
                } else {
                    break;
                }
            }
            return (Vec::from(ans), None);
        }
        *offset = std::cmp::min(*offset, self.select.unwrap_or(*offset));
        if !self.list.is_match(*offset) {
            *offset = match self.list.next(*offset) {
                Some(x) => x,
                None => self.list.previous(*offset).unwrap(),
            }
        }
        let mut count = *offset;
        loop{
            if ans.len() >= height as usize {
                if count > self.select.unwrap_or(0){
                    break
                }
                ans.pop_front();
            }
            ans.push_back(count);
            count = if let Some(x) = self.list.next(count){x}else{break}
        }
        while ans.len() > height as usize{
            ans.pop_front();
        }
        *offset = ans[0];
        let select = if let Some(x) = self.select{
            ans.iter().position(|&i|i==x)
        }else{
            None
        };
        (Vec::from(ans), select)
    }

    pub fn add_packet(&mut self, p: Packet) {
        self.list.push(p);
    }

    pub fn next(&mut self) {
        if self.select.is_none() {
            self.select = self.list.filterd_last();
            return;
        }
        if let Some(x) = self.list.next(self.select.unwrap()){
         self.select=   Some(x)
        };
    }

    pub fn previous(&mut self) {
        if self.select.is_none() {
            self.select = self.list.filterd_first();
            return;
        }
        if let Some(x) = self.list.previous(self.select.unwrap()){
         self.select=   Some(x)
        };
    }

    pub fn to_view(&mut self) {
        self.view = self.select;
    }

    pub fn unselect(&mut self) {
        self.select = None;
    }

    pub fn next_forcus(&mut self) {
        self.input_mode.next()
    }
    pub fn get_input_mode(&self) -> InputMode {
        self.input_mode
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InputMode {
    Filter,
    List,
    View,
}

impl InputMode {
    fn next(&mut self) {
        match self {
            InputMode::Filter => *self = InputMode::List,
            InputMode::List => *self = InputMode::View,
            InputMode::View => *self = InputMode::Filter,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::read_packet;
    #[test]
    fn it_works() {
        let mut app = App::default();

        let tcp_packet = [
            0x61u8, 0x6u8, 0xadu8, 0x63u8, 0x85u8, 0x5cu8, 0x2u8, 0x00u8, 0xa4u8, 0x00u8, 0x00u8,
            0x00u8, 0xa4u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0xf6u8, 0x20u8, 0x8au8, 0x1cu8, 0xa6u8,
            0xe0u8, 0x3fu8, 0x49u8, 0x86u8, 0xfu8, 0x9fu8, 0x8u8, 0x00u8, 0x45u8, 0x00u8, 0x00u8,
            0x96u8, 0x82u8, 0xabu8, 0x40u8, 0x00u8, 0x80u8, 0x6u8, 0x00u8, 0x00u8, 0xc0u8, 0xa8u8,
            0xbu8, 0xau8, 0xc0u8, 0xa8u8, 0xbu8, 0x3u8, 0xe5u8, 0xb5u8, 0x1fu8, 0x49u8, 0x9cu8,
            0x91u8, 0x77u8, 0x97u8, 0x60u8, 0x4eu8, 0x1bu8, 0xb2u8, 0x50u8, 0x18u8, 0x1u8, 0xffu8,
            0x97u8, 0xe6u8, 0x00u8, 0x00u8, 0x17u8, 0x3u8, 0x3u8, 0x00u8, 0x69u8, 0x4au8, 0xd5u8,
            0x61u8, 0xb8u8, 0x2du8, 0x43u8, 0x94u8, 0x3au8, 0xc2u8, 0xafu8, 0x00u8, 0xd4u8, 0x7fu8,
            0x7u8, 0xbeu8, 0xd1u8, 0x4cu8, 0x63u8, 0x32u8, 0x4eu8, 0xa4u8, 0x78u8, 0xb9u8, 0x2au8,
            0xf4u8, 0xc1u8, 0xf3u8, 0xdau8, 0xfdu8, 0x3u8, 0xfau8, 0xfcu8, 0x4au8, 0x46u8, 0x00u8,
            0x6au8, 0xc2u8, 0x16u8, 0xb7u8, 0xe1u8, 0xecu8, 0xc7u8, 0x7u8, 0x14u8, 0x49u8, 0xb5u8,
            0xc8u8, 0x4cu8, 0xbau8, 0x2du8, 0xcfu8, 0xa1u8, 0x8du8, 0x72u8, 0x58u8, 0x52u8, 0x3bu8,
            0x99u8, 0x58u8, 0xf4u8, 0xcu8, 0x5du8, 0x24u8, 0x42u8, 0x77u8, 0x78u8, 0x73u8, 0xafu8,
            0x46u8, 0x97u8, 0xe8u8, 0xe8u8, 0x1fu8, 0xc7u8, 0x3bu8, 0x46u8, 0x77u8, 0x49u8, 0x45u8,
            0x2u8, 0x1du8, 0x53u8, 0x37u8, 0x70u8, 0x38u8, 0x4bu8, 0x99u8, 0x17u8, 0x8eu8, 0x66u8,
            0x88u8, 0x8fu8, 0xb3u8, 0xb2u8, 0x28u8, 0xc5u8, 0x94u8, 0x2cu8, 0x3bu8, 0x3au8, 0xbcu8,
            0x6fu8, 0xa8u8, 0xfdu8, 0x78u8,
        ];

        let udp_packet = [
            0x21u8, 0x21u8, 0xadu8, 0x63u8, 0x59u8, 0xcdu8, 0x5u8, 0x00u8, 0x5cu8, 0x00u8, 0x00u8,
            0x00u8, 0x5cu8, 0x00u8, 0x00u8, 0x00u8, 0x1u8, 0x00u8, 0x5eu8, 0x00u8, 0x00u8, 0x1u8,
            0xc4u8, 0x3cu8, 0xeau8, 0x6u8, 0xddu8, 0x00u8, 0x8u8, 0x00u8, 0x45u8, 0x00u8, 0x00u8,
            0x4eu8, 0xd9u8, 0xa6u8, 0x40u8, 0x00u8, 0x1u8, 0x11u8, 0xe5u8, 0x4du8, 0xc0u8, 0xa8u8,
            0xbu8, 0x1u8, 0xefu8, 0x00u8, 0x00u8, 0x1u8, 0xebu8, 0x70u8, 0x17u8, 0x70u8, 0x00u8,
            0x3au8, 0x6cu8, 0xa2u8, 0x74u8, 0x69u8, 0x6du8, 0x65u8, 0x20u8, 0x69u8, 0x73u8, 0x20u8,
            0x54u8, 0x68u8, 0x75u8, 0x20u8, 0x44u8, 0x65u8, 0x63u8, 0x20u8, 0x32u8, 0x39u8, 0x20u8,
            0x31u8, 0x34u8, 0x3au8, 0x30u8, 0x39u8, 0x3au8, 0x35u8, 0x32u8, 0x20u8, 0x32u8, 0x30u8,
            0x32u8, 0x32u8, 0x20u8, 0x5bu8, 0x6cu8, 0x6fu8, 0x6fu8, 0x70u8, 0x5fu8, 0x64u8, 0x65u8,
            0x74u8, 0x65u8, 0x63u8, 0x74u8, 0x69u8, 0x6fu8, 0x6eu8, 0x5du8, 0x00u8,
        ];

        async_std::task::block_on(async {
            for i in 0..20 {
                let mut packet_read = if i % 2 == 0 {
                    &tcp_packet[..]
                } else {
                    &udp_packet[..]
                };
                let packet = read_packet(&mut packet_read).await;
                if let Ok(packet) = packet {
                    app.add_packet(packet);
                }
            }
        });
        let mut offset = 0;
        let height = 5;
        let (view_list, _) = app.get_view_list(height, &mut offset);
        assert_eq!(view_list.len(),height as usize);
        app.list.add_filter_str("UDP");
        let (view_list, _) = app.get_view_list(height, &mut offset);
        assert_eq!(view_list.len(),height as usize);
        app.next();
        let (_, select) = app.get_view_list(height, &mut offset);
        assert_eq!(select, Some(4));
        app.previous();
        let (_, select) = app.get_view_list(height, &mut offset);
        assert_eq!(select, Some(3));
        app.previous();
        app.previous();
        app.previous();
        app.previous();
        let (_view_list, select) = app.get_view_list(height, &mut offset);
        assert_eq!(select, Some(0));
        app.next();
        let (_view_list, select) = app.get_view_list(height, &mut offset);
        assert_eq!(select, Some(1));
    }
}
