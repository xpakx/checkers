CREATE TABLE game (
   id BIGINT GENERATED BY DEFAULT AS IDENTITY NOT NULL,

   invitation SMALLINT NOT NULL,
   game_type SMALLINT NOT NULL,
   ruleset SMALLINT NOT NULL,
   ai_type SMALLINT NOT NULL,
   status SMALLINT NOT NULL,

   current_state VARCHAR(255) NOT NULL,
   user_starts BOOLEAN NOT NULL,
   user_turn BOOLEAN NOT NULL,

   user_id BIGINT NOT NUll,
   opponent_id BIGINT,
   CONSTRAINT pk_game PRIMARY KEY (id),
   CONSTRAINT fk_user_id
      FOREIGN KEY(user_id)
      REFERENCES account(id),
   CONSTRAINT fk_opponent_id
      FOREIGN KEY(opponent_id)
      REFERENCES account(id)
);