# Dumb Relational Database (DRD)

https://en.wikipedia.org/wiki/Relational_algebra#Introduction

https://cs186berkeley.net/notes/note6/

## Syntax

(Lowest to highest precedence)

```
Let         var = exp
Select      var ... <- exp
Where       exp ? exp
Union       exp + exp
Difference  exp - exp
Product     exp * exp
Or          exp | exp
Equals      exp == exp
And         exp & exp
Not         !exp
Parens      (exp)
Table       [exp ...]
Row         {var exp ...}
Bool        true
Int         -42
Str         'hi'
Var         x
```

## Operations

### Let

```
var = exp;
```

### Row

```
Alice = {name "Alice" title "Product Manager"};
```

### Table

```
Person = [{name "Alice"   title "Product Manager"   level 1}
          {name "Bob"     title "Software Engineer" level 2}
          {name "Charlie" title "Software Engineer" level 1}];
```

### Projection (π)

SQL: `SELECT`

Example: `SELECT column FROM table;`

```
column <- table;
```

### Selection (σ)

SQL: `WHERE`

Example: `SELECT * FROM table WHERE condition;`

```
* <- table ? condition;
```

### Cartesian product (×)

SQL: `A, B` or `CROSS JOIN`

Example: `SELECT * FROM table1, table2;`

```
* <- table1 * table2;
```

### Union (∪)

SQL: `UNION` 

Example: `SELECT * FROM table1 UNION SELECT * FROM table2;`

```
(* <- table1) + (* <- table2);
```

### Set difference (-)

SQL: `EXCEPT` or `MINUS`

Example: `SELECT * FROM table1 EXCEPT SELECT * FROM table2;`

```
(* <- table1) - (* <- table2);
```

### Rename (ρ)

(Not added yet)

SQL: `AS`

Example: `SELECT column AS new_name FROM table;`

```
(column := new_name) <- table;
```
