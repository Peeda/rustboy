# Notes about graphics
## Intro
* Graphics are managed in 8x8 tiles, not as individual pixels
    * Two bits each, corresponding to a color as defined by the palette
    * For an object 0 means transparent and doesn't refer to a color
* Background, Window, then Objects are drawn back to front
    * Objects are either 8x8 or 8x16 (two tiles vertically)
    * Sprites are several objects combined to draw a larger object
## VRAM Tile Data
* 8000-97FF are reserved for tiles, enough for 384, 3 blocks of 128
* 8 bit indexing, either 8000 or 9000 is the base and offset is signed or unsigned
depending on the LCDC register
* one method indexes the first two blocks, the other takes the second two
* Each tile is 16 bytes, 2 bytes for each row of the image
    * That's 16 bits for 8 pixels, each pixel being two bits
