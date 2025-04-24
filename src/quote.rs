pub struct Quote {
   pub words: &'static str,
   pub author: &'static str,
}

pub const THE_QUOTE: Quote = Quote {
   words: "\"For if joyful is the fountain that rises in the sun, 
   its springs are in the wells of sorrow unfathomable at the foundations of the Earth.\"",
   author: "--J.R.R. Tolkien, The Silmarillion",
};