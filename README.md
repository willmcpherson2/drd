# Dumb Relational Database (DRD)

https://en.wikipedia.org/wiki/Relational_algebra#Introduction

https://cs186berkeley.net/notes/note6/

## Syntax

```
let        (var = exp)

table      [exp ...]
row        {var exp ...}

select     (var ... <- exp)
where      (exp ? exp)
product    (exp * exp)
union      (exp + exp)
difference (exp - exp)

equals     (exp == exp)
and        (exp & exp)
or         (exp | exp)
not        (! exp)

bool       true
int        -42
str        "hi"
var        x
```

## Operations

### Let

`(var = exp)`

### Row

`(Alice = {name "Alice" title "Product Manager"})`

### Table

```
(Person = [{name "Alice"   title "Product Manager"   level 1}
           {name "Bob"     title "Software Engineer" level 2}
           {name "Charlie" title "Software Engineer" level 1}])
```

### Projection (π)

SQL: `SELECT`

Example: `SELECT column FROM table;`

`(column <- table)`

### Selection (σ)

SQL: `WHERE`

Example: `SELECT * FROM table WHERE condition;`

`(* <- (table ? condition))`

### Cartesian product (×)

SQL: `A, B` or `CROSS JOIN`

Example: `SELECT * FROM table1, table2;`

`(* <- (table1 * table2))`

### Union (∪)

SQL: `UNION` 

Example: `SELECT * FROM table1 UNION SELECT * FROM table2;`

`((* <- table1) + (* <- table2))`

### Set difference (-)

SQL: `EXCEPT` or `MINUS`

Example: `SELECT * FROM table1 EXCEPT SELECT * FROM table2;`

`((* <- table1) - (* <- table2))`

### Rename (ρ)

SQL: `AS`

Example: `SELECT column AS new_name FROM table;`

`((column := new-name) <- table)`
