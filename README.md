# geoterm

> [!WARNING]
> This project is in an extremely early stage and completely non-functional.

geoterm is, first and foremost, geoguessr: terminal edition. That's pretty much it.

As a secondary goal, there should also be a rudimentary serial client that can be used
to play on effectively any serial device with a little work, like a gameboy or calculator.

## Plans

* Varied levels of image quality for different devices.
* Simple 2x zoom functionality (again to accomodate low resolution).
* OCR in multiple languages to accompany lower quality image data.

## Communication Protocol Details

Here, all communication is done through raw TCP. [msgpack](https://msgpack.org/index.html) is used
to encode the data, since it's language agnostic. In addition, all packets begin with a 4 byte length,
where all numbers in the entire packet are in big-endian.
