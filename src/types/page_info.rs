/*****************************************************************************
MIT License

Copyright (c) 2024 Bruce Skingle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
******************************************************************************/

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ForwardPageInfo {
    pub end_cursor: String,
    pub has_next_page: bool
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ReversePageInfo {
    pub start_cursor: String,
    pub has_previous_page: bool
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub start_cursor: String,
    pub has_next_page: bool,
    pub end_cursor: String,
    pub has_previous_page: bool
}

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct Query<P,V> {
//     pub params: P,
//     pub value: V
// }

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct ForwardPageParams {
//     pub after: String,
//     pub first: Int
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForwardPageOf<T> 
{
    pub page_info: ForwardPageInfo,
    pub edges: Vec<EdgeOf<T>>
}

impl<T> IntoIterator for ForwardPageOf<T> {
    type Item = T;

    type IntoIter = ForwardPageOfIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        ForwardPageOfIterator {
          iter: self.edges.into_iter(),
        }
    }
}

pub struct ForwardPageOfIterator<T> {
  iter: std::vec::IntoIter<EdgeOf<T>>
}

impl<T> Iterator for ForwardPageOfIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| e.node)
    }
}

impl<'a, T> IntoIterator for &'a ForwardPageOf<T> {
  type Item = &'a T;

  type IntoIter = RefForwardPageOfIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    // let iter: std::slice::Iter<'a, EdgeOf<T>> = self.edges.iter();
      RefForwardPageOfIterator {
        iter: self.edges.iter(),
      }
  }
}

pub struct RefForwardPageOfIterator<'a, T> {
  iter: std::slice::Iter<'a, EdgeOf<T>>
}

impl<'a, T> Iterator for RefForwardPageOfIterator<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
      self.iter.next().map(|e| &e.node)
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReversePageOf<T> 
{
    pub page_info: ReversePageInfo,
    pub edges: Vec<EdgeOf<T>>
}

impl<T> IntoIterator for ReversePageOf<T> {
    type Item = T;

    type IntoIter = ReversePageOfIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        ReversePageOfIterator {
          iter: self.edges.into_iter(),
        }
    }
}

pub struct ReversePageOfIterator<T> {
  iter: std::vec::IntoIter<EdgeOf<T>>
}

impl<T> Iterator for ReversePageOfIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| e.node)
    }
}

impl<'a, T> IntoIterator for &'a ReversePageOf<T> {
  type Item = &'a T;

  type IntoIter = RefReversePageOfIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    // let iter: std::slice::Iter<'a, EdgeOf<T>> = self.edges.iter();
      RefReversePageOfIterator {
        iter: self.edges.iter(),
      }
  }
}

pub struct RefReversePageOfIterator<'a, T> {
  iter: std::slice::Iter<'a, EdgeOf<T>>
}

impl<'a, T> Iterator for RefReversePageOfIterator<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
      self.iter.next().map(|e| &e.node)
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageOf<T> 
{
    pub page_info: PageInfo,
    pub edges: Vec<EdgeOf<T>>
}

impl<T> IntoIterator for PageOf<T> {
    type Item = T;

    type IntoIter = PageOfIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        PageOfIterator {
          iter: self.edges.into_iter(),
        }
    }
}

pub struct PageOfIterator<T> {
  iter: std::vec::IntoIter<EdgeOf<T>>
}

impl<T> Iterator for PageOfIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| e.node)
    }
}

impl<'a, T> IntoIterator for &'a PageOf<T> {
  type Item = &'a T;

  type IntoIter = RefPageOfIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    // let iter: std::slice::Iter<'a, EdgeOf<T>> = self.edges.iter();
      RefPageOfIterator {
        iter: self.edges.iter(),
      }
  }
}

pub struct RefPageOfIterator<'a, T> {
  iter: std::slice::Iter<'a, EdgeOf<T>>
}

impl<'a, T> Iterator for RefPageOfIterator<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
      self.iter.next().map(|e| &e.node)
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct  EdgeOf<T>
{
  pub node: T,
  // pub cursor: String
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let page = PageOf {
          page_info: PageInfo {
              start_cursor: String::from("back"),
              has_next_page: false,
              end_cursor: String::from("forward"),
              has_previous_page: false,
          },
          edges: vec!(EdgeOf { node: 1}, EdgeOf { node: 2}, EdgeOf { node: 3})
        };

        for i in &page {
          println!("i={}", i);
        }

        for i in page {
          println!("i2={}", i);
        }

        // panic!("test")
    }
}