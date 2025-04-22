pub struct Quote {
   pub words: &'static str,
   pub author: &'static str,
}

pub const THE_QUOTE: Quote = Quote {
   words: "some wise words",
   author: "the author",
};