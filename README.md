# The Shadowbox Database

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

Boolean     true
Integer     -42
String      'hi'
Variable    x
```

## Example

Run this with:

```
cargo run -- run examples/main.sdb
```

```
--------------------------------------------------------------------------------
-- Let
-- Syntax: variable = expression ; body
--------------------------------------------------------------------------------

thing = (x = true; not true); -- false

--------------------------------------------------------------------------------
-- Table
-- Syntax: variables : expressions
-- SQL: CREATE TABLE combined with INSERT INTO
--------------------------------------------------------------------------------

Staff =
  id, name, employed :
  1, 'Alice', true,
  2, 'Bob', true,
  3, 'Charlie', false;

-- id, name, employed :
-- 1, 'Alice', true,
-- 2, 'Bob', true,
-- 3, 'Charlie', false;

--------------------------------------------------------------------------------
-- Select
-- Syntax: variables <- table
-- SQL: SELECT
-- Relational algebra: Projection (π)
--------------------------------------------------------------------------------

Names = id, name <- Staff;

-- id, name :
-- 1, 'Alice',
-- 2, 'Bob',
-- 3, 'Charlie';

--------------------------------------------------------------------------------
-- Where
-- Syntax: table ? condition
-- SQL: WHERE
-- Relational algebra: Selection (σ)
--------------------------------------------------------------------------------

Alice = Staff ? employed && name == 'Alice';

-- id, name, employed :
-- 1, 'Alice', true,

--------------------------------------------------------------------------------
-- Union
-- Syntax: table + table
-- SQL: UNION
-- Relational algebra: Union (∪)
--------------------------------------------------------------------------------

David = id, name, employed : 4, 'David', true;
Staff = Staff + David;

-- id, name, employed :
-- 1, 'Alice', true,
-- 2, 'Bob', true,
-- 3, 'Charlie', false;
-- 4, 'David', true;

--------------------------------------------------------------------------------
-- Difference
-- Syntax: table - table
-- SQL: MINUS or EXCEPT
-- Relational algebra: Set difference (-)
--------------------------------------------------------------------------------

Staff = Staff - David;

-- id, name, employed :
-- 1, 'Alice', true,
-- 2, 'Bob', true,
-- 3, 'Charlie', false;

--------------------------------------------------------------------------------
-- Product
-- Syntax: table * table
-- SQL: CROSS JOIN
-- Relational algebra: Cartesian product (×)
--------------------------------------------------------------------------------

Names = name <- Staff;
Pairs = Names * Names;

-- name, name :
-- 'Alice', 'Alice',
-- 'Alice', 'Bob',
-- 'Alice', 'Charlie',
-- 'Bob', 'Alice',
-- 'Bob', 'Bob',
-- 'Bob', 'Charlie',
-- 'Charlie', 'Alice',
-- 'Charlie', 'Bob',
-- 'Charlie', 'Charlie';

--------------------------------------------------------------------------------
-- Booleans
--------------------------------------------------------------------------------

Results =
  result :
  true,
  not false,
  true || false,
  true && true,
  1 == 1;

-- result :
-- true
-- true
-- true
-- true
-- true

main = Results;

main -- the body of our big let expression!
```
