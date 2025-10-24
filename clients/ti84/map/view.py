from PIL import Image
import sys

def main(input_file):
    width, height = 320, 240

    with open(input_file, "rb") as f:
        data = f.read()

    img = Image.new("RGBA", (width, height))
    pixels = img.load()

    def decode(n):
        if n == 0b00:
            return 0
        elif n == 0b01:
            return 64
        elif n == 0b10:
            return 128
        else:  # 0b11
            return 255

    idx = 0
    for y in range(height):
        for x_block in range(0, width, 4):
            byte = data[idx]
            idx += 1
            a_values = [(byte >> 6) & 0b11, (byte >> 4) & 0b11, (byte >> 2) & 0b11, byte & 0b11]
            for i, a in enumerate(a_values):
                if x_block + i < width:
                    pixels[x_block + i, y] = (255, 255, 255, decode(a))

    img.show()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python view.py output.bin")
        sys.exit(1)
    main(sys.argv[1])