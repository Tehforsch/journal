import shutil
import os
from pathlib import Path

def rewrite(f):
    with open(f, "r") as g:
        lines = g.readlines()
    assert "#+TITLE:" in lines[0]
    assert "file:20200320142808-journal.org" in lines[1]
    with open(f, "w") as f:
        f.writelines(lines[2:])
        

def main():
    for f in os.listdir("data"):
        f = "data" / Path(f)
        _,rest = f.name.split("-")
        _,year,month,day = rest.replace(".org","").split("_")
        folder = Path(Path("output") / f"{year}-{month}-{day}")
        folder.mkdir(exist_ok=True)

        target = folder / "entry.md"
        source = f

        shutil.copy(source, target)

        rewrite(target)

main()
