pub struct Quote {
   pub quote: &'static str,
   pub author: &'static str,
}

pub const THE_QUOTE: Quote = Quote {
   quote: "some wise words",
   author: "the author",
};