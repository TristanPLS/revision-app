-- Génération IA des notes Cornell et des schémas (dual coding) : combler les
-- dernières "zones d'ombre" où l'IA n'avait pas la main.
ALTER TYPE job_kind ADD VALUE IF NOT EXISTS 'cornell';
ALTER TYPE job_kind ADD VALUE IF NOT EXISTS 'schema';
