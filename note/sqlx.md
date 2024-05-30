
```sh
docker run --name postgres \
    -p 5432:5432 \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -v /opt/docker/pgdata:/var/lib/postgresql/data \
    -d postgres:latest



docker exec -it postgres bash
psql -U postgres

```

好的，以下是 `psql` 的简单使用教程，涵盖如何连接到数据库以及如何查询数据库和数据表。

### 连接到 PostgreSQL 数据库

#### 1. 通过命令行连接
假设你已经启动了 PostgreSQL 容器，并且知道数据库的连接信息。使用以下命令连接到 PostgreSQL 数据库：

```bash
docker exec -it postgres bash
psql -U postgres

CREATE DATABASE shortener;

```

在这里：
- `-h localhost` 指定主机地址为 `localhost`。
- `-p 5432` 指定端口号为 `5432`。
- `-U postgres` 指定用户名为 `postgres`。
- `-W` 提示输入密码。

输入密码后，你将进入 `psql` 交互式终端。

### 查询数据库和数据表

#### 2. 列出所有数据库
在 `psql` 交互式终端中，使用以下命令列出所有数据库：

```sql
\l
```

或者：

```sql
\list
```

#### 3. 连接到特定数据库
假设你想连接到名为 `mydatabase` 的数据库，使用以下命令：

```sql
\c mydatabase
```

或者：

```sql
\connect mydatabase
```

#### 4. 列出所有数据表
在连接到特定数据库后，使用以下命令列出该数据库中的所有数据表：

```sql
\dt
```

如果你想查看特定模式下的所有表，可以使用：

```sql
\dt schema_name.*
```

#### 5. 查看数据表结构
使用以下命令查看特定数据表的结构：

```sql
\d table_name
```

#### 6. 查询数据表中的数据
使用标准的 SQL 查询语句查询数据表中的数据。例如，查询名为 `users` 的数据表中的所有记录：

```sql
SELECT * FROM users;
```

#### 7. 插入数据
使用 `INSERT` 语句向数据表中插入数据。例如，向 `users` 表中插入一条记录：

```sql
INSERT INTO users (name, email) VALUES ('John Doe', 'john.doe@example.com');
```

#### 8. 更新数据
使用 `UPDATE` 语句更新数据表中的记录。例如，更新 `users` 表中 `name` 为 `John Doe` 的用户的 `email`：

```sql
UPDATE users SET email = 'john.doe@newdomain.com' WHERE name = 'John Doe';
```

#### 9. 删除数据
使用 `DELETE` 语句删除数据表中的记录。例如，删除 `users` 表中 `name` 为 `John Doe` 的用户：

```sql
DELETE FROM users WHERE name = 'John Doe';
```

#### 10. 退出 `psql`
使用以下命令退出 `psql` 交互式终端：

```sql
\q
```

### 总结
以上是 `psql` 的基本使用教程，涵盖了连接到数据库、列出数据库和数据表、查询数据表结构和数据等操作。希望这些内容能帮助你顺利使用 `psql` 进行数据库操作。
