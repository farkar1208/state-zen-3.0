# StateZen DSL 语法规则（Whitespace-Delimited）

## 顶层结构
```
<program> ::= <aspect-section> <zone-section> <transition-section>

<aspect-section> ::= "Aspect" <newline> (<aspect-decl> <newline>)*
<zone-section>   ::= "Zone" <newline> (<zone-decl> <newline>)*
<transition-section> ::= "Transition" <newline> (<transition-decl> <newline> (<indented-line> <newline>)*)*
```

## 基础元素
```
<ident>        ::= [a-zA-Z_][a-zA-Z0-9_]*
<int-lit>      ::= [0-9]+
<float-lit>    ::= [0-9]+\.[0-9]+
<bool-lit>     ::= "true" | "false"
<newline>      ::= "\n"
<indent>       ::= "  "  // exactly two spaces
```

## Aspect 声明（三列：name, type-spec, default）
```
<aspect-decl> ::= <indent> <ident> <whitespace> <type-spec> <whitespace> <literal>
<type-spec>   ::= 
    "bool"
  | "enum" "{" <enum-lit> ("," <enum-lit>)* "}"
  | <num-lit> "<=" <scalar-type> "<=" <num-lit>
  | <num-lit> "<"  <scalar-type> "<=" <num-lit>
  | <num-lit> "<=" <scalar-type> "<"  <num-lit>
  | <num-lit> "<"  <scalar-type> "<"  <num-lit>

<scalar-type> ::= "u8" | "u16" | "u32" | "i8" | "i16" | "f32" | "f64"
<enum-lit>    ::= <ident>
<literal>     ::= <int-lit> | <float-lit> | <bool-lit> | <enum-lit>
<whitespace>  ::= " "+  // one or more spaces
```

## Zone 声明（两列：name, predicate）
```
<zone-decl> ::= <indent> <ident> <whitespace> <predicate>
<predicate> ::= <term> (" AND " <term>)* | <term> (" OR " <term>)*
<term>      ::= <atom> | "NOT " <atom>
<atom>      ::= 
    <ident> "==" <literal>
  | <ident> "!=" <literal>
  | <ident> "<" <literal>
  | <ident> "<=" <literal>
  | <ident> ">" <literal>
  | <ident> ">=" <literal>
  | "(" <predicate> ")"
```

## Transition 声明
```
<transition-decl> ::= <indent> <ident> <whitespace> <event-id>
<event-id>        ::= <ident>
<indented-line>   ::= <indent><indent> <field-name> <whitespace> <field-value>
<field-name>      ::= "ActiveIn" | "Update" | "OnTran"
<field-value>     ::= 
    <predicate>                     // for ActiveIn
  | <update-stmt>                   // for Update
  | <side-effect-ref>               // for OnTran

<update-stmt>     ::= <assign> (";" <assign>)* ";"
<assign>          ::= <ident> ":=" <expr>
<expr>            ::= 
    <literal>
  | <ident>
  | <expr> "+" <expr>
  | <expr> "-" <expr>
  | "min(" <expr> "," <expr> ")"
  | "max(" <expr> "," <expr> ")"
  | "clamp(" <expr> "," <literal> "," <literal> ")"

<side-effect-ref> ::= <ident> ("(" <literal> ("," <literal>)* ")")?
```