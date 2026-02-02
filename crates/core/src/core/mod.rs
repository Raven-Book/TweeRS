pub mod parser;
mod skip;
/**
Twine Spec: https://github.com/iftechfoundation/twine-specs

Supports Twee3 file format and Twine 2 story format only.
HTML output is generated directly through Twee3 struct without entity struct.
*/
pub mod story;

pub mod output;
