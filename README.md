# Dumb Relational Database (DRD)

## Operators

(Highest to lowest precedence)

| Operator   | Syntax              |
|------------|---------------------|
| Column     | `name: type`        |
| Table      | `column, column`    |
| Equality   | `column == data`    |
| Where      | `table ? condition` |
| Select     | `column <- table`   |
| Let        | `name = expression` |

## Tables

A table can be either:

1. An "entity table" containing only an ID column
2. A "data table" containing an ID column and a data column, holding scalar values
3. A "relation table" containing an ID column and a foreign ID column, referencing another table

Employee (entity table)

```
Employee = id: Id
```

| id |
|----|
| 1  |
| 2  |
| 3  |

Customer (entity table)

```
Customer = id: Id
```

| id |
|----|
| 4  |
| 5  |

Phone (data table)

```
Phone = id: Id, phone: String
```

| id | phone: String |
|----|---------------|
| 1  | 0417483829    |
| 2  | 0471837400    |
| 3  | 0403718282    |
| 4  | 0473838838    |
| 5  | 0417208374    |

Manager (relation table)

```
Manager = id: Id, manager: Employee
```

| id | manager: Employee |
|----|-------------------|
| 2  | 1                 |
| 3  | 2                 |

## Queries

```
id <- Employee -- The identity function, funnily enough
id <- Phone -- The IDs of the Phone table
phone <- Phone -- The phone numbers in the Phone table

'foo' == 'foo' -- true
1 == 2 -- false

Phone ? true -- Identity function
Phone ? false -- Empty
Phone ? phone == '0417483829' -- Table with one phone number. `phone` is in scope.

-- What's my manager's phone number?
phone <- Phone ? id == (manager <- Manager ? id == 3)
```
