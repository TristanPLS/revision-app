-- "Tout générer d'un coup" : un job orchestrant blocs + flashcards + examen +
-- Feynman + carte conceptuelle à partir d'un plan d'étude proposé par l'IA.
-- (Postgres 16 : ADD VALUE est autorisé en transaction tant qu'on ne l'utilise
--  pas dans la même migration — c'est le cas ici.)
ALTER TYPE job_kind ADD VALUE IF NOT EXISTS 'bundle';
