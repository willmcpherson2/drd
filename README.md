# The Shadowbox Database

Shadowbox (sdb) is a simple relational database. It implements the [relational algebra](https://en.wikipedia.org/wiki/Relational_algebra#Introduction).

sdb has no `CREATE TABLE` or `INSERT` statements. Instead, it has table literals and **variable shadowing**.

```
Staff =
  id, name, employed :
  1, 'Alice', true,
  2, 'Bob', true;
Staff = Staff + id, name, employed : 3, 'Charlie', false;
Staff
```

In this example, we first define a `Staff` table. Then we re-define it as the union of the old table and a new table. Then we put `Staff` in the body of the let expression to query it.

```
$ sdb run examples/charlie.sdb
id, name, employed : 1, 'Alice', true, 2, 'Bob', true, 3, 'Charlie', false
```

However, this is not persistent.

To persist our staff table, we first need to start the database server:

```
$ sdb start
Starting server
Directory: db
http://localhost:2345
```

Now we can set up our staff table:

```
Staff =
  id, name, employed :
  1, 'Alice', true,
  2, 'Bob', true;
Staff
```

```
$ sdb run -s localhost:2345 examples/staff.sdb
id, name, employed : 1, 'Alice', true, 2, 'Bob', true
```

It will persist in human-readable form:

```
$ cat db/Staff 
id, name, employed : 1, 'Alice', true, 2, 'Bob', true
```

Now we can re-define `Staff` persistently:

```
Staff = Staff + id, name, employed : 3, 'Christian', true;
Staff
```

```
$ sdb run -s localhost:2345 examples/christian.sdb 
id, name, employed : 1, 'Alice', true, 2, 'Bob', true, 3, 'Christian', true
```

How did that work? It's equivalent to our first example. When you define a variable, the server writes it to disk. When you reference a variable, the server reads it from disk. This means that variable shadowing works across connections.

## Syntax

```
exp
  var = exp; exp
  vars <- exp
  exp ? exp
  exp + exp
  exp - exp
  exp * exp
  vars : exps
  exp || exp
  exp && exp
  exp == exp
  not exp
  bool
  int
  str
  var

vars
  var, vars
  var
  nil

exps
  exp, exps
  exp
  nil
```
