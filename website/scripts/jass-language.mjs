// Small, project-authored TextMate grammar for JASS declarations and examples.
export default {
  name: "jass",
  displayName: "JASS",
  scopeName: "source.jass",
  patterns: [
    { name: "comment.line.double-slash.jass", match: "//.*$" },
    {
      name: "string.quoted.double.jass",
      begin: '"',
      end: '"',
      patterns: [{ name: "constant.character.escape.jass", match: "\\\\." }],
    },
    { name: "string.quoted.single.jass", match: "'[^']*'" },
    {
      name: "keyword.control.jass",
      match:
        "\\b(?:type|extends|constant|native|function|endfunction|takes|returns|globals|endglobals|local|set|call|if|then|else|elseif|endif|loop|endloop|exitwhen|return|and|or|not|array)\\b",
    },
    { name: "constant.language.jass", match: "\\b(?:true|false|null)\\b" },
    {
      name: "storage.type.jass",
      match:
        "\\b(?:integer|real|boolean|string|code|handle|nothing|unit|player|trigger|timer|group|location)\\b",
    },
    { name: "constant.numeric.jass", match: "\\b(?:0[xX][0-9a-fA-F]+|[0-9]+(?:\\.[0-9]*)?)\\b" },
    { name: "entity.name.function.jass", match: "\\b[A-Za-z_][A-Za-z0-9_]*(?=\\s*\\()" },
  ],
};
