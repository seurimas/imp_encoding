# Impractical(-ish) Encoding

This crate contains some sister code to a series of blog posts I am writing at https://write.as/comments-and-code. I share these encoding methods in the hopes that someone might find them interesting or even useful for their own creative projects. Each module comes with a small set of tests and examples to demonstrate their usage. None of these modules are very practical, as the encoded size will be a great deal larger than a more sensible encoding method, like Base64. Encoding is **NOT** encryption, and encoded values can be easily recovered from the encoded text; apply your own encryption for uses which require some sort of secrecy.

## Modules

This crate is divided into several modules, with feature switches by the same name to enable them.
* futhark - Serialize values into Strings of ancient runes.