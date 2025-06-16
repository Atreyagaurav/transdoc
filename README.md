# Translation Doc

Tool to generate translation document from a custom syntax text file.

The text file consists of sentences like this:

    @ line-label
    This is a << sentence = word meaning >> in original language.
    ---
    This here will be a full translation of the above sentence
	
The word meanings are there so people can associate the meanings to the words, while not having a "not-literal" translation for the whole sentence.

Refer to files with `.chapter` extension to see a full working prototypes.


This is a prototype based on an idea that should help language learners. Future plan includes:
- Multiple Languages support,
- dictionary loading,
- Plugin system to auto-translation,
- audio file for "speak aloud" funcionality,
- interactive hide/show translations,
- APIs to collect the word meanings to make quizes, etc.

