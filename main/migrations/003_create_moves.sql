CREATE TABLE move (
   id BIGINT GENERATED BY DEFAULT AS IDENTITY NOT NULL,
   "x" INTEGER,
   "y" INTEGER,
   current_state VARCHAR(255),
   timestamp TIME,
   game_id BIGINT NOT NULL,
   CONSTRAINT pk_move PRIMARY KEY (id),
   CONSTRAINT fk_game_id
      FOREIGN KEY(game_id)
      REFERENCES game(id)
);
