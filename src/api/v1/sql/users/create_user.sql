INSERT INTO users (name, email, password, role, verified) VALUES ($1, $2, $3, $4, $5) RETURNING *;

