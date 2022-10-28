# XML to CSV converter for StackOverflow post data

A small script that follows [this blog post](https://docs.rs/quick-xml/0.26.0/quick_xml/reader/struct.Reader.html#method.read_event_into) in terms of implementation details. 

The script's purpose is to parse the Internet Archve [StackOverflow post data](https://archive.org/details/stackexchange) from XML into CSV. 

NB: Yes, I know that SO data is available as a public dataset in Google BigQuery. I am doing this mostly as an exercise in Rust/scripting.