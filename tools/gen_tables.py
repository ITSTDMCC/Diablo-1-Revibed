"""Generate the game's static data tables (src/tables/*.rs) from the DevilutionX 1.5.3 sources.

  python tools/gen_tables.py

Each spec names the C++ array, the Rust struct (fields in declaration order, with Rust types and
how each initializer is converted) and default values for trailing members the C++ leaves out.
Field names keep the C++ spelling so ported code reads like the original.
"""
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
SRC = os.path.join(CRATE, '..', 'Decomp', 'source_1.5.3', 'Source')


# ---------------------------------------------------------------- parsing

def strip_comments(s):
    out = []
    i = 0
    while i < len(s):
        if s.startswith('//', i):
            j = s.find('\n', i)
            i = len(s) if j < 0 else j
        elif s.startswith('/*', i):
            i = s.index('*/', i) + 2
            out.append(' ')
        elif s[i] == '"':
            j = i + 1
            while s[j] != '"':
                j += 2 if s[j] == '\\' else 1
            out.append(s[i:j + 1])
            i = j + 1
        else:
            out.append(s[i])
            i += 1
    return ''.join(out)


def array_body(text, decl):
    i = text.index(decl)
    i = text.index('{', i)
    depth = 0
    j = i
    while j < len(text):
        c = text[j]
        if c == '"':
            j += 1
            while text[j] != '"':
                j += 2 if text[j] == '\\' else 1
        elif c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return text[i:j + 1]
        j += 1
    raise ValueError(decl)


def parse_init(s):
    """Parse a brace initializer into nested lists; leaves are expression strings."""
    pos = 0

    def skip_ws():
        nonlocal pos
        while pos < len(s) and s[pos].isspace():
            pos += 1

    def parse_list():
        nonlocal pos
        assert s[pos] == '{'
        pos += 1
        items = []
        while True:
            skip_ws()
            if s[pos] == '}':
                pos += 1
                return items
            if s[pos] == '{':
                items.append(parse_list())
            else:
                items.append(parse_expr())
            skip_ws()
            if s[pos] == ',':
                pos += 1
            elif s[pos] == '}':
                continue
            else:
                raise ValueError(s[pos - 40:pos + 40])

    def parse_expr():
        nonlocal pos
        start = pos
        depth = 0
        while True:
            c = s[pos]
            if c == '"':
                pos += 1
                while s[pos] != '"':
                    pos += 2 if s[pos] == '\\' else 1
            elif c in '(<' and not (c == '<' and s[pos:pos + 2] == '<<'):
                if c == '(' or s[start:pos].rstrip().endswith(('static_cast', 'enum_size')):
                    depth += 1
            elif c in ')>' and depth and not (c == '>' and s[pos:pos + 2] == '>>'):
                depth -= 1
            elif depth == 0 and c in ',}':
                return s[start:pos].strip()
            elif c == '<' and s[pos:pos + 2] == '<<':
                pos += 1
            elif c == '>' and s[pos:pos + 2] == '>>':
                pos += 1
            pos += 1

    skip_ws()
    return parse_list()


# ---------------------------------------------------------------- conversion

ALIASES = {}


def join_literals(e):
    """C adjacent string literals "a" "b" -> "ab"."""
    lits = re.findall(r'"((?:[^"\\]|\\.)*)"', e)
    rest = re.sub(r'"((?:[^"\\]|\\.)*)"', '', e).strip()
    if lits and rest == '':
        return '"' + ''.join(lits) + '"'
    return e


def conv_expr(e, ty):
    e = e.strip()
    m = re.fullmatch(r'(N_|_|P_)\((.*)\)', e, flags=re.S)
    if m:
        inner = m.group(2)
        if m.group(1) == 'P_':
            inner = inner.split(',', 1)[1]
        e = m.group(1) + '(' + join_literals(inner.strip()) + ')' if m.group(1) != 'P_' else join_literals(inner.strip())
    e = join_literals(e)
    m = re.fullmatch(r'(?:N_|_)\(\s*("(?:[^"\\]|\\.)*")\s*\)', e)
    if m:
        e = m.group(1)
    m = re.fullmatch(r'P_\(\s*"([^"]*)"\s*,\s*("(?:[^"\\]|\\.)*")\s*\)', e)
    if m:
        e = m.group(2)
    if ty.startswith('Option<'):
        if e == 'nullptr':
            return 'None'
        return f'Some({conv_expr(e, ty[7:-1])})'
    if e == 'nullptr':
        return '""' if ty == "&'static str" else 'None'
    m = re.fullmatch(r'static_cast<int>\(\s*([\d.]+)F\s*\*\s*(\d+)\s*\)', e)
    if m:
        cast = ty if re.fullmatch(r'[iu](8|16|32|64)', ty) else 'i32'
        return f'({m.group(1)}f32 * {m.group(2)}.0) as {cast}'
    e = re.sub(r'static_cast<[^>]+>\(([^()]*)\)', r'(\1)', e)
    e = re.sub(r'\b(\d+)[uU]\b', r'\1', e)
    for a, b in ALIASES.items():
        e = re.sub(r'(?<![\w:])' + a + r'(?![\w:])', b, e)
    return e


def conv_flags(e, ty):
    parts = [conv_expr(p, ty) for p in e.split('|')]
    if len(parts) == 1:
        return parts[0]
    return f'{ty}(' + ' | '.join(p + '.0' for p in parts) + ')'


class Struct:
    def __init__(self, name, fields, derive='Clone, Copy, Debug'):
        self.name = name
        self.fields = fields  # (name, rust type, kind, default)
        self.derive = derive


STRUCTS = {}


def struct(name, fields, derive='Clone, Copy, Debug'):
    STRUCTS[name] = Struct(name, fields, derive)


def conv(value, ty, kind, default):
    if kind.startswith('struct:'):
        return conv_struct(STRUCTS[kind[7:]], value if value is not None else [])
    if kind.startswith('arr:'):
        _, n, inner_kind = kind.split(':', 2)
        inner_ty = re.fullmatch(r'\[(.*); \d+\]', ty).group(1)
        vals = value if value is not None else []
        items = [conv(vals[i] if i < len(vals) else None, inner_ty, inner_kind, None) for i in range(int(n))]
        return '[' + ', '.join(items) + ']'
    if value is None:
        if default is None:
            raise ValueError(f'missing value without default for {ty}')
        return default
    if value == []:
        # `{}`: value-initialised
        if kind == 'flags':
            return f'{ty}(0)'
        if ty == 'bool':
            return 'false'
        if ty == "&'static str":
            return '""'
        if ty.startswith('Option<'):
            return 'None'
        return '0'
    if isinstance(value, list):
        raise ValueError(f'unexpected braces for {ty}: {value}')
    if kind == 'flags':
        return conv_flags(value, ty)
    out = conv_expr(value, ty)
    if re.fullmatch(r'[iu](8|16|32|64)', ty) and re.search(r'[A-Za-z_]', out) and ' as ' not in out:
        out = f'({out}) as {ty}'  # C++ integer conversions are implicit
    return out


def conv_struct(st, values):
    parts = []
    for i, (fname, fty, kind, default) in enumerate(st.fields):
        v = values[i] if i < len(values) else None
        parts.append(f'{fname}: {conv(v, fty, kind, default)}')
    if len(values) > len(st.fields):
        raise ValueError(f'too many values for {st.name}: {values}')
    return f'{st.name} {{ ' + ', '.join(parts) + ' }'


def emit_struct(st):
    lines = [f'#[derive({st.derive})]', f'pub struct {st.name} {{']
    for fname, fty, kind, default in st.fields:
        lines.append(f'    pub {fname}: {fty},')
    lines.append('}')
    return '\n'.join(lines)


def table(text, decl, rust_name, st, count=None):
    body = parse_init(array_body(text, decl))
    rows = [conv_struct(st, r) for r in body]
    n = count or len(rows)
    return f'pub static {rust_name}: [{st.name}; {n}] = [\n' + ''.join(f'    {r},\n' for r in rows) + '];', len(rows)


def flat_table(text, decl, rust_name, ty, kind='v'):
    body = parse_init(array_body(text, decl))

    def walk(v):
        if isinstance(v, list):
            return '[' + ', '.join(walk(x) for x in v) + ']'
        return conv(v, ty, kind, None)

    def shape(v):
        if isinstance(v, list):
            return f'[{shape(v[0])}; {len(v)}]'
        return ty

    return f'pub static {rust_name}: {shape(body)} = {walk(body)};'


def read(rel):
    return strip_comments(open(os.path.join(SRC, rel), encoding='utf-8').read())


def write(name, header, parts):
    path = os.path.join(CRATE, 'src', 'tables', name)
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(header + '\n\n' + '\n\n'.join(parts) + '\n')


HEADER = ('//! Generated by `tools/gen_tables.py` from `Source/{src}` (DevilutionX 1.5.3). Do not edit.\n\n'
          '#![allow(non_snake_case, non_upper_case_globals, unused_imports, clippy::all)]\n\n'
          'use crate::effects_data::*;\nuse crate::enums::*;')


# ---------------------------------------------------------------- tables

def gen_playerdat():
    t = read('playerdat.cpp')
    u8 = 'u8'
    struct('PlayerData', [
        ('className', "&'static str", 'v', None), ('classPath', "&'static str", 'v', None),
        ('baseStr', u8, 'v', None), ('baseMag', u8, 'v', None), ('baseDex', u8, 'v', None), ('baseVit', u8, 'v', None),
        ('maxStr', u8, 'v', None), ('maxMag', u8, 'v', None), ('maxDex', u8, 'v', None), ('maxVit', u8, 'v', None),
        ('blockBonus', u8, 'v', None), ('adjLife', 'i16', 'v', None), ('adjMana', 'i16', 'v', None),
        ('lvlLife', 'i16', 'v', None), ('lvlMana', 'i16', 'v', None), ('chrLife', 'i16', 'v', None),
        ('chrMana', 'i16', 'v', None), ('itmLife', 'i16', 'v', None), ('itmMana', 'i16', 'v', None),
        ('skill', 'SpellID', 'v', None)])
    struct('PlayerSpriteData', [(n, u8, 'v', None) for n in
                                'stand walk attack bow swHit block lightning fire magic death'.split()])
    struct('PlayerAnimData', [(n, 'i8', 'v', None) for n in (
        'unarmedFrames unarmedActionFrame unarmedShieldFrames unarmedShieldActionFrame swordFrames swordActionFrame '
        'swordShieldFrames swordShieldActionFrame bowFrames bowActionFrame axeFrames axeActionFrame maceFrames '
        'maceActionFrame maceShieldFrames maceShieldActionFrame staffFrames staffActionFrame idleFrames walkingFrames '
        'blockingFrames deathFrames castingFrames recoveryFrames townIdleFrames townWalkingFrames castingActionFrame'
    ).split()])
    parts = [emit_struct(STRUCTS[n]) for n in ('PlayerData', 'PlayerSpriteData', 'PlayerAnimData')]
    parts.append(flat_table(t, 'ExpLvlsTbl[MaxCharacterLevel] =', 'ExpLvlsTbl', 'u32'))
    parts.append(flat_table(t, 'herosounds[enum_size<HeroClass>::value][enum_size<HeroSpeech>::value] =', 'herosounds', 'SfxId'))
    for decl, name, st in (('PlayersData[] =', 'PlayersData', 'PlayerData'), ('PlayersSpriteData[] =', 'PlayersSpriteData', 'PlayerSpriteData'),
                           ('PlayersAnimData[] =', 'PlayersAnimData', 'PlayerAnimData')):
        parts.append(table(t, decl, name, STRUCTS[st])[0])
    write('playerdat.rs', HEADER.format(src='playerdat.cpp'), parts)


def gen_spelldat():
    t = read('spelldat.cpp')
    for a in ('Fire', 'Lightning', 'Magic', 'Targeted', 'AllowedInTown'):
        ALIASES[a] = 'SpellDataFlags::' + a
    struct('SpellData', [
        ('sNameText', "&'static str", 'v', None), ('sSFX', 'SfxId', 'v', None), ('bookCost10', 'u16', 'v', None),
        ('staffCost10', 'u8', 'v', None), ('sManaCost', 'u8', 'v', None), ('flags', 'SpellDataFlags', 'flags', None),
        ('sBookLvl', 'i8', 'v', None), ('sStaffLvl', 'i8', 'v', None), ('minInt', 'u8', 'v', None),
        ('sMissiles', '[MissileID; 2]', 'arr:2:v', None), ('sManaAdj', 'u8', 'v', None), ('sMinMana', 'u8', 'v', None),
        ('sStaffMin', 'u8', 'v', None), ('sStaffMax', 'u8', 'v', None)])
    parts = [emit_struct(STRUCTS['SpellData']), table(t, 'SpellsData[] =', 'SpellsData', STRUCTS['SpellData'])[0]]
    ALIASES.clear()
    write('spelldat.rs', HEADER.format(src='spelldat.cpp'), parts)


def gen_itemdat():
    t = read('itemdat.cpp')
    struct('ItemData', [
        ('iRnd', 'item_drop_rate', 'v', None), ('iClass', 'item_class', 'v', None), ('iLoc', 'item_equip_type', 'v', None),
        ('iCurs', 'item_cursor_graphic', 'v', None), ('itype', 'ItemType', 'v', None), ('iItemId', 'unique_base_item', 'v', None),
        ('iName', "&'static str", 'v', None), ('iSName', "Option<&'static str>", 'v', None), ('iMinMLvl', 'u8', 'v', None),
        ('iDurability', 'u8', 'v', None), ('iMinDam', 'u8', 'v', None), ('iMaxDam', 'u8', 'v', None),
        ('iMinAC', 'u8', 'v', None), ('iMaxAC', 'u8', 'v', None), ('iMinStr', 'u8', 'v', None), ('iMinMag', 'u8', 'v', None),
        ('iMinDex', 'u8', 'v', None), ('iFlags', 'ItemSpecialEffect', 'flags', None), ('iMiscId', 'item_misc_id', 'v', None),
        ('iSpell', 'SpellID', 'v', None), ('iUsable', 'bool', 'v', None), ('iValue', 'u16', 'v', None)])
    struct('ItemPower', [('type_', 'item_effect_type', 'v', 'IPL_INVALID'), ('param1', 'i32', 'v', '0'), ('param2', 'i32', 'v', '0')],
           derive='Clone, Copy, Debug, PartialEq, Eq')
    struct('PLStruct', [
        ('PLName', "&'static str", 'v', None), ('power', 'ItemPower', 'struct:ItemPower', None), ('PLMinLvl', 'i8', 'v', None),
        ('PLIType', 'AffixItemType', 'flags', None), ('PLGOE', 'goodorevil', 'v', None), ('PLDouble', 'bool', 'v', None),
        ('PLOk', 'bool', 'v', None), ('minVal', 'i32', 'v', None), ('maxVal', 'i32', 'v', None), ('multVal', 'i32', 'v', None)])
    struct('UniqueItem', [
        ('UIName', "&'static str", 'v', None), ('UIItemId', 'unique_base_item', 'v', None), ('UIMinLvl', 'i8', 'v', None),
        ('UINumPL', 'u8', 'v', None), ('UIValue', 'i32', 'v', None), ('powers', '[ItemPower; 6]', 'arr:6:struct:ItemPower', None)])
    parts = [emit_struct(STRUCTS[n]) for n in ('ItemData', 'ItemPower', 'PLStruct', 'UniqueItem')]
    for decl, name, st in (('AllItemsList[] =', 'AllItemsList', 'ItemData'), ('ItemPrefixes[] =', 'ItemPrefixes', 'PLStruct'),
                           ('ItemSuffixes[] =', 'ItemSuffixes', 'PLStruct'), ('UniqueItems[] =', 'UniqueItems', 'UniqueItem')):
        parts.append(table(t, decl, name, STRUCTS[st])[0])
    write('itemdat.rs', HEADER.format(src='itemdat.cpp'), parts)


def gen_monstdat():
    t = read('monstdat.cpp')
    struct('MonsterData', [
        ('name', "&'static str", 'v', None), ('assetsSuffix', "&'static str", 'v', None),
        ('soundSuffix', "Option<&'static str>", 'v', None), ('trnFile', "Option<&'static str>", 'v', None),
        ('availability', 'MonsterAvailability', 'v', None), ('width', 'u16', 'v', None), ('image', 'u16', 'v', None),
        ('hasSpecial', 'bool', 'v', None), ('hasSpecialSound', 'bool', 'v', None), ('frames', '[i8; 6]', 'arr:6:v', None),
        ('rate', '[i8; 6]', 'arr:6:v', None), ('minDunLvl', 'i8', 'v', None), ('maxDunLvl', 'i8', 'v', None),
        ('level', 'i8', 'v', None), ('hitPointsMinimum', 'u16', 'v', None), ('hitPointsMaximum', 'u16', 'v', None),
        ('ai', 'MonsterAIID', 'v', None), ('abilityFlags', 'u16', 'v', None), ('intelligence', 'u8', 'v', None),
        ('toHit', 'u8', 'v', None), ('animFrameNum', 'i8', 'v', None), ('minDamage', 'u8', 'v', None),
        ('maxDamage', 'u8', 'v', None), ('toHitSpecial', 'u8', 'v', None), ('animFrameNumSpecial', 'i8', 'v', None),
        ('minDamageSpecial', 'u8', 'v', None), ('maxDamageSpecial', 'u8', 'v', None), ('armorClass', 'u8', 'v', None),
        ('monsterClass', 'MonsterClass', 'v', None), ('resistance', 'u8', 'v', None), ('resistanceHell', 'u8', 'v', None),
        ('selectionType', 'i8', 'v', None), ('treasure', 'u16', 'v', None), ('exp', 'u16', 'v', None)])
    struct('UniqueMonsterData', [
        ('mtype', '_monster_id', 'v', None), ('mName', "&'static str", 'v', None), ('mTrnName', "Option<&'static str>", 'v', None),
        ('mlevel', 'u8', 'v', None), ('mmaxhp', 'u16', 'v', None), ('mAi', 'MonsterAIID', 'v', None), ('mint', 'u8', 'v', None),
        ('mMinDamage', 'u8', 'v', None), ('mMaxDamage', 'u8', 'v', None), ('mMagicRes', 'u16', 'v', None),
        ('monsterPack', 'UniqueMonsterPack', 'v', None), ('customToHit', 'u8', 'v', None), ('customArmorClass', 'u8', 'v', None),
        ('mtalkmsg', '_speech_id', 'v', None)])
    parts = ['/// Returns a `treasure` value for the given item (`Uniq`).\npub const fn Uniq(item: _unique_items) -> u16 {\n    T_UNIQ + item as u16\n}']
    parts += [emit_struct(STRUCTS[n]) for n in ('MonsterData', 'UniqueMonsterData')]
    parts.append(table(t, 'MonstersData[] =', 'MonstersData', STRUCTS['MonsterData'])[0])
    parts.append(flat_table(t, 'MonstConvTbl[] =', 'MonstConvTbl', '_monster_id'))
    parts.append(table(t, 'UniqueMonstersData[] =', 'UniqueMonstersData', STRUCTS['UniqueMonsterData'])[0])
    write('monstdat.rs', HEADER.format(src='monstdat.cpp'), parts)


def gen_objdat():
    t = read('objdat.cpp')
    for a in ('Animated', 'Solid', 'MissilesPassThrough', 'Light', 'Trap', 'Breakable'):
        ALIASES[a] = 'ObjectDataFlags::' + a
    struct('ObjectData', [
        ('ofindex', 'object_graphic_id', 'v', None), ('minlvl', 'i8', 'v', None), ('maxlvl', 'i8', 'v', None),
        ('olvltype', 'i8', 'v', None), ('otheme', 'theme_id', 'v', None), ('oquest', 'quest_id', 'v', None),
        ('flags', 'ObjectDataFlags', 'flags', None), ('animDelay', 'u8', 'v', None), ('animLen', 'u8', 'v', None),
        ('animWidth', 'u8', 'v', None), ('selFlag', 'i8', 'v', None)])
    parts = ['use crate::levels::gendung::dtype::*;', emit_struct(STRUCTS['ObjectData'])]
    parts.append(flat_table(t, 'ObjTypeConv[] =', 'ObjTypeConv', '_object_id'))
    parts.append(table(t, 'AllObjects[109] =', 'AllObjects', STRUCTS['ObjectData'])[0])
    parts.append(flat_table(t, 'ObjMasterLoadList[] =', 'ObjMasterLoadList', "&'static str"))
    ALIASES.clear()
    write('objdat.rs', HEADER.format(src='objdat.cpp'), parts)


def gen_textdat():
    t = read('textdat.cpp')
    struct('Speech', [('txtstr', "&'static str", 'v', None), ('scrlltxt', 'bool', 'v', None), ('sfxnr', 'SfxId', 'v', None)])
    parts = [emit_struct(STRUCTS['Speech']), table(t, 'Speeches[] =', 'Speeches', STRUCTS['Speech'])[0]]
    write('textdat.rs', HEADER.format(src='textdat.cpp'), parts)


def gen_items_tables():
    t = read('items.cpp')
    parts = [
        flat_table(t, 'ItemCAnimTbl[] =', 'ItemCAnimTbl', 'i8'),
        flat_table(t, 'ItemInvSnds[] =', 'ItemInvSnds', 'SfxId'),
        flat_table(t, 'OilLevels[] =', 'OilLevels', 'i32'),
        flat_table(t, 'OilValues[] =', 'OilValues', 'i32'),
        flat_table(t, 'OilMagic[] =', 'OilMagic', 'item_misc_id'),
        flat_table(t, 'OilNames[10][25] =', 'OilNames', "&'static str"),
        flat_table(t, 'ItemDropNames[] =', 'ItemDropNames', "&'static str"),
        flat_table(t, 'ItemAnimLs[] =', 'ItemAnimLs', 'i8'),
        flat_table(t, 'ItemDropSnds[] =', 'ItemDropSnds', 'SfxId'),
        flat_table(t, 'premiumlvladd[] =', 'premiumlvladd', 'i32'),
        flat_table(t, 'premiumLvlAddHellfire[] =', 'premiumLvlAddHellfire', 'i32'),
    ]
    write('items_tables.rs', HEADER.format(src='items.cpp'), parts)


def main():
    os.makedirs(os.path.join(CRATE, 'src', 'tables'), exist_ok=True)
    gen_playerdat()
    gen_spelldat()
    gen_itemdat()
    gen_monstdat()
    gen_objdat()
    gen_textdat()
    gen_items_tables()
    print('ok')


if __name__ == '__main__':
    main()
