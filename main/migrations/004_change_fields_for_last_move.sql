ALTER TABLE move DROP COLUMN IF EXISTS "x";
ALTER TABLE move DROP COLUMN IF EXISTS "y";
ALTER TABLE move ADD last_move VARCHAR(255);
