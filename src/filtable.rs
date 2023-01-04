pub trait Filtable {
    fn is_match(&self, s: &str) -> bool;
}

#[derive(Debug)]
struct FiltableListItem<T: Filtable> {
    item: T,
    previous: Option<usize>,
    next: Option<usize>,
}

#[derive(Debug)]
pub struct FiltableList<T: Filtable> {
    list: Vec<FiltableListItem<T>>,
    filter: String,
}

impl<T: Filtable> FiltableList<T> {
    pub fn new() -> Self {
        Self {
            list: vec![],
            filter: "".to_string(),
        }
    }
    pub fn is_empty(&self) -> bool{
        self.list.is_empty()
    }
    pub fn is_filterd_empty(&self) -> bool{
        if self.list.is_empty() == true{
            return true
        }
        if self.next(0) == None{
            return true
        }
        return false
    }
    pub fn get_filter(&self) -> String{
        self.filter.clone()
    }
    pub fn get_item(&self, i: usize) -> &T{
        &self.list[i].item
    }
    pub fn is_match(&self, i:usize) -> bool{
        self.list[i].item.is_match(&self.filter)
    }
    pub fn len(&self) -> usize{
        self.list.len()
    }
    pub fn push(&mut self, item: T) {
        let last = if let Some(x) = self.list.last() {
            x
        } else {
            let item = FiltableListItem {
                item,
                previous: None,
                next: None,
            };
            self.list.push(item);
            return;
        };
        if last.item.is_match(&self.filter) {
            let item = FiltableListItem {
                item,
                previous: Some(self.list.len() - 1),
                next: None,
            };
            self.list.push(item);
        } else {
            let item = FiltableListItem {
                item,
                previous: last.previous,
                next: None,
            };
            self.list.push(item);
        }
        if !self.list[self.list.len() - 1].item.is_match(&self.filter) {
            return;
        }
        let mut count = self.list.len() - 2;
        while self.list[count].next == None {
            self.list[count].next = Some(self.list.len() - 1);
            if count > 0{
                count -=1
            }else{
                break
            }
        }
    }
    fn update_filter(&mut self) {
        let mut last_true = None;
        for i in 0..self.list.len() {
            self.list[i].previous = last_true;
            if self.list[i].item.is_match(&self.filter) {
                last_true = Some(i)
            }
        }
        last_true = None;
        for i in (0..self.list.len()).rev() {
            self.list[i].next = last_true;
            if self.list[i].item.is_match(&self.filter) {
                last_true = Some(i);
            }
        }
    }

    pub fn add_filter_str(&mut self, st: &str) {
        self.filter.push_str(st);
        self.update_filter();
    }

    pub fn delete_filter_char(&mut self) {
        self.filter.pop();
        self.update_filter();
    }
    pub fn filterd_first(&self) -> Option<usize> {
        if self.list.is_empty() {
            return None;
        }
        if self.list[0].item.is_match(&self.filter) {
            return Some(0);
        }
        self.list[0].next
    }

    pub fn filterd_last(&self) -> Option<usize> {
        if self.list.is_empty() {
            return None;
        }
        if self.list[self.list.len() - 1].item.is_match(&self.filter) {
            return Some(self.list.len() - 1);
        }
        self.list[self.list.len() - 1].previous
    }
    pub fn next(&self, i: usize) -> Option<usize> {
        self.list[i].next
    }
    pub fn previous(&self, i: usize) -> Option<usize> {
        self.list[i].previous
    }
}

impl<T: Filtable> Default for FiltableList<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Mock {
        s: String,
    }
    impl Filtable for Mock {
        fn is_match(&self, s: &str) -> bool {
            self.s.contains(s)
        }
    }

    #[test]
    fn it_work() {
        let mut li = FiltableList::<Mock>::default();
        assert_eq!(li.filterd_first(), None);
        assert_eq!(li.filterd_last(), None);
        for i in 0..20 {
            if i % 2 == 0 {
                li.push(Mock {
                    s: "tcp".to_string(),
                });
            } else {
                li.push(Mock {
                    s: "udp".to_string(),
                });
            }
        }
        assert_eq!(li.filterd_first(), Some(0));
        assert_eq!(li.filterd_last(), Some(19));
        li.add_filter_str("tcp");
        assert_eq!(li.filterd_first(), Some(0));
        assert_eq!(li.filterd_last(), Some(18));
        li.push(Mock {
            s: "ucp".to_string(),
        });
        li.delete_filter_char();
        li.delete_filter_char();
        li.delete_filter_char();
        li.add_filter_str("udp");
        assert_eq!(li.filterd_first(), Some(1));
        assert_eq!(li.next(1), Some(3));
        assert_eq!(li.next(0), Some(1));
        assert_eq!(li.previous(3), Some(1));
    }
}
