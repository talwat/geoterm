use std::ops::Index;

use crate::ClientOptions;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Clients {
    inner: Vec<Client>,
}

impl Index<usize> for Clients {
    type Output = Client;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<'a> IntoIterator for &'a Clients {
    type Item = &'a Client;
    type IntoIter = std::slice::Iter<'a, Client>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Clients {
    type Item = &'a mut Client;
    type IntoIter = std::slice::IterMut<'a, Client>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl Clients {
    pub fn empty() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn into_inner(self) -> Vec<Client> {
        self.inner
    }
}

impl From<Vec<Client>> for Clients {
    fn from(value: Vec<Client>) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Client {
    pub id: usize,
    pub ready: bool,
    pub options: ClientOptions,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Action {
    Join = 0,
    Return,
    Leave,
    Ready,
}
