from PIL import Image
import sys

def main(input, output):
    img = Image.open(input).convert("RGBA")
    if img.size != (320, 240):
        img = img.resize((320, 240), Image.LANCZOS)

    pixels = img.load()
    data = bytearray()

    def classify(a):
        if a == 0:
            return 0b00
        elif a < 85:
            return 0b01
        elif a < 170:
            return 0b10
        else:
            return 0b11

    for y in range(img.height):
        row = []
        for x in range(img.width):
            r, g, b, a = pixels[x, y]
            row.append(classify(a))
            if len(row) == 4:
                byte = (row[0] << 6) | (row[1] << 4) | (row[2] << 2) | row[3]
                data.append(byte)
                row.clear()

        if row:
            while len(row) < 4:
                row.append(0)
            byte = (row[0] << 6) | (row[1] << 4) | (row[2] << 2) | row[3]
            data.append(byte)

    with open(output, "wb") as f:
        f.write(data)

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python map.py input.png output.raw")
        sys.exit(1)
    main(sys.argv[1], sys.argv[2])
