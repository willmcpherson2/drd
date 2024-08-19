# Dumb Relational Database (DRD)

https://en.wikipedia.org/wiki/Relational_algebra#Introduction

https://cs186berkeley.net/notes/note6/

## Syntax

(Lowest to highest precedence)

```
Let         var = exp; exp

Select      var,* <- exp
Where       exp ? exp
Union       exp + exp
Difference  exp - exp
Product     exp * exp

Table       var,* : exp,*

Or          exp || exp
And         exp && exp
Equals      exp == exp
Not         not exp

Bool        true
Int         -42
Str         'hi'
Var         x
```

## Operations

### Projection (π)

SQL: `SELECT`

```
columns <- table
```

### Selection (σ)

SQL: `WHERE`

```
table ? condition
```

### Union (∪)

SQL: `UNION` 

```
table1 + table2
```

### Set difference (-)

SQL: `MINUS` or `EXCEPT`

```
table1 - table2
```

### Cartesian product (×)

SQL: `CROSS JOIN` or `A, B`

```
table1 * table2
```
