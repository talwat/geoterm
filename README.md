# geoterm

> [!WARNING]
> This project is in an extremely early stage and completely non-functional.

geoterm is, first and foremost, geoguessr: calculator edition. That's pretty much it.

As a secondary goal, there should also be a rudimentary terminal client that can be used
to play on PC.

## Plans/Specifics

* OCR in multiple languages to accompany lower quality image data.
* 4-bit color at 320x240 for the calculator screen.
* Black & white guesser window with simple world map.
* Potentially dumping raw C struct data & using microcontroller to translate into msgpack?

## Communication Protocol Details

Here, all communication is done through raw TCP. [msgpack](https://msgpack.org/index.html) is used
to encode the data, since it's language agnostic. In addition, all packets begin with a 4 byte length,
where all numbers in the entire packet are in big-endian.
