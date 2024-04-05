# Observer prototype

This file does a really stupid high frequency observer for data that you
provide it.

Basically, pass it a regex and either point it at a file or point a
stream at it, and it'll collect up the most commonly occurring elements.
At the end of the stream, it'll dump out what it found.

For now: `python3 observer --help` for args.

(You shouldn't need to install any dependencies yet.)

TODO: make this a webservice, or provide some sort of active reporting.
