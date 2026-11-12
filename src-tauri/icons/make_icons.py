"""Generate simple original TopCalc icons. Not a Desmos or GeoGebra asset."""
import math
import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent


def write_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = b"".join(b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    path.write_bytes(png)


def render(size: int) -> bytes:
    pixels = bytearray(size * size * 4)
    for y in range(size):
        for x in range(size):
            nx = (x + 0.5) / size
            ny = (y + 0.5) / size
            dx = nx - 0.5
            dy = ny - 0.5
            radius = math.hypot(dx, dy)
            i = (y * size + x) * 4
            if radius > 0.46:
                continue
            pixels[i : i + 4] = bytes((22, 24, 32, 255))
            wave = math.sin(nx * math.tau)
            wave_y = 0.5 - wave * 0.22
            if abs(ny - wave_y) < 0.035:
                pixels[i : i + 4] = bytes((110, 168, 254, 255))
    return bytes(pixels)


def write_ico(path: Path, images: list[tuple[int, bytes]]) -> None:
    count = len(images)
    header = struct.pack("<HHH", 0, 1, count)
    offset = 6 + 16 * count
    entries = b""
    blob = b""
    for size, png in images:
        entries += struct.pack("<BBBBHHII", size if size < 256 else 0, size if size < 256 else 0, 0, 0, 1, 32, len(png), offset)
        offset += len(png)
        blob += png
    path.write_bytes(header + entries + blob)


def write_icns(path: Path, chunks: list[tuple[bytes, bytes]]) -> None:
    body = b"".join(kind + struct.pack(">I", len(data) + 8) + data for kind, data in chunks)
    path.write_bytes(b"icns" + struct.pack(">I", len(body) + 8) + body)


def main() -> None:
    sizes = {
        32: OUT / "32x32.png",
        128: OUT / "128x128.png",
        256: OUT / "128x128@2x.png",
    }
    pngs: dict[int, bytes] = {}
    for size, path in sizes.items():
        data = render(size)
        write_png(path, size, size, data)
        pngs[size] = path.read_bytes()
    for extra in (16, 64, 512, 1024):
        data = render(extra)
        tmp = OUT / f"_{extra}.png"
        write_png(tmp, extra, extra, data)
        pngs[extra] = tmp.read_bytes()
        tmp.unlink()
    write_ico(OUT / "icon.ico", [(16, pngs[16]), (32, pngs[32]), (64, pngs[64]), (128, pngs[128]), (256, pngs[256])])
    write_icns(
        OUT / "icon.icns",
        [
            (b"icp4", pngs[16]),
            (b"icp5", pngs[32]),
            (b"icp6", pngs[64]),
            (b"ic07", pngs[128]),
            (b"ic08", pngs[256]),
            (b"ic09", pngs[512]),
            (b"ic10", pngs[1024]),
        ],
    )


if __name__ == "__main__":
    main()
