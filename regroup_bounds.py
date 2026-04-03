import os
import re

TRAITS = [
    "",
    "From",
    "TryInto",
    "Add",
    "AddAssign",
    "BitAnd",
    "BitAndAssign",
    "BitOr",
    "BitOrAssign",
    "BitXor",
    "BitXorAssign",
    "Div",
    "DivAssign",
    "Mul",
    "MulAssign",
    "Pow",
    "Rem",
    "RemAssign",
    "Shl",
    "ShlAssign",
    "Shr",
    "ShrAssign",
    "Sub",
    "SubAssign",
]

def update_lines(lines):
    start = lines.index("where\n")
    end = lines.index("{\n", start)
    bounds = [line for line in lines[start+1:end] if "//" not in line]
    bound_groups = {trait: [] for trait in TRAITS}
    for line in bounds:
        matching_trait = ""
        for trait in TRAITS:
            if trait and re.search(rf"\b{trait}\b", line):
                matching_trait = trait
                break
        bound_groups[matching_trait].append(line)
    for group_lines in bound_groups.values():
        group_lines.sort()
    bounds = []
    for trait, group_lines in bound_groups.items():
        if trait:
            bounds.append(f"    // {trait} bounds\n")
        bounds.extend(group_lines)
    return lines[:start+1] + uniq(bounds) + lines[end:]

def uniq(lst):
    result = []
    for item in lst:
        if item not in result:
            result.append(item)
    return result

def main():
    os.chdir(os.path.dirname(__file__))

    for file in [
        "src/big_integer.rs",
        "src/big_number.rs",
        "src/big_natural.rs",
    ]:
        print(f"Updating {file}...")
        with open(file, "r") as f:
            lines = f.readlines()
        lines = update_lines(lines)
        with open(file, "w") as f:
            f.writelines(lines)

if __name__ == "__main__":
    main()