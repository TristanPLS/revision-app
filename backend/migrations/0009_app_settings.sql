-- Réglages d'instance (BYOK) : une seule ligne, créée au premier enregistrement
-- depuis la page Réglages. Les colonnes NULL retombent sur les variables
-- d'environnement puis sur les défauts du provider.
-- NOTE : la clé API est stockée EN CLAIR dans la base (instance mono-utilisateur
-- auto-hébergée — documenté dans le README, section Sécurité).
CREATE TABLE app_settings (
    id          smallint PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    ai_provider text,
    ai_model    text,
    ai_base_url text,
    ai_api_key  text,
    updated_at  timestamptz NOT NULL DEFAULT now()
);
