-- =====================================================================
--  revision-app — jeu de données de DÉMONSTRATION
--
--  Un cours d'exemple « La Guerre froide » entièrement généré par l'IA :
--  34 flashcards, un examen blanc de 15 questions, 7 concepts Feynman,
--  une fiche Cornell, une carte conceptuelle et des schémas.
--
--  Il permet d'explorer l'application immédiatement, SANS configurer la
--  moindre clé IA — toutes les cartes sont « à réviser » dès le chargement.
--
--  CHARGER (après `docker compose up -d`) :
--    docker compose exec -T postgres psql -U revision revision < scripts/demo-seed.sql
--
--  RETIRER : supprime la matière « Guerre froide (démo) » depuis l'application
--  (la suppression efface en cascade cartes, examen, carte conceptuelle, etc.).
--
--  À charger UNE SEULE FOIS (les identifiants sont fixes ; un second
--  chargement provoquerait des erreurs de clés dupliquées, sans gravité).
--
--  Ce fichier ne contient AUCUN secret (ni clé API, ni réglages).
-- =====================================================================

--
-- PostgreSQL database dump
--

\restrict kZu4nAxJIactEFwRhAFoWvqGNajslNMSkSkELVeq9itWtuutAr1DfCXtc41aUCd

-- Dumped from database version 16.14
-- Dumped by pg_dump version 16.14

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: subjects; Type: TABLE DATA; Schema: public; Owner: -
--

SET SESSION AUTHORIZATION DEFAULT;

ALTER TABLE public.subjects DISABLE TRIGGER ALL;

COPY public.subjects (id, name, description, exam_date, created_at, updated_at) FROM stdin;
514807e5-57aa-4a7e-a3cc-1f8b725a4051	Guerre froide (démo)	Cours d'exemple généré par l'IA — supprimable en un clic.	\N	2026-06-05 18:37:02.208676+00	2026-06-05 18:37:02.208676+00
\.


ALTER TABLE public.subjects ENABLE TRIGGER ALL;

--
-- Data for Name: blocks; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.blocks DISABLE TRIGGER ALL;

COPY public.blocks (id, subject_id, code, title, summary, "position", created_at) FROM stdin;
55b1a5e7-52bf-403e-8c1d-fab628127618	514807e5-57aa-4a7e-a3cc-1f8b725a4051	B1	Origines et Formation des Blocs	L'émergence d'un monde bipolarisé entre le bloc occidental (USA) et le bloc oriental (bloc soviétique) suite aux désaccords post-Seconde Guerre mondiale.	0	2026-06-05 18:37:26.701468+00
5700703a-5246-4cf1-b66c-b98343b666e3	514807e5-57aa-4a7e-a3cc-1f8b725a4051	B2	Crises et Tensions	Une période d'affrontements indirects et de course aux armements, marquée par unsustainable tension et des crises majeures comme Berlin et Cuba.	1	2026-06-05 18:37:26.704081+00
a03287a6-426b-4eb3-b5d6-43b332f95fad	514807e5-57aa-4a7e-a3cc-1f8b725a4051	B3	Coexistence Pacifique et Détente	Une phase d'apaisement relatif et de négociations diplomatiques pour limiter les armements et stabilisation des frontières.	2	2026-06-05 18:37:26.706089+00
f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	514807e5-57aa-4a7e-a3cc-1f8b725a4051	B4	Fin de la Guerre Froide	L'effondrement économique et politique de l'URSS et la chute des régimes communistes, menant à l'unipolarité américaine.	3	2026-06-05 18:37:26.707653+00
11d63919-6e8d-4a92-bbf2-f521964a5b44	514807e5-57aa-4a7e-a3cc-1f8b725a4051	B5	Introduction et Conclusion	Définition de la guerre froide, concept de dissuasion nucléaire et bilan final de l'Héritage.	4	2026-06-05 18:37:26.710469+00
0ef6a2e9-0f80-4ef8-a681-5802e94f0f86	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	Ideologies and Alliances	Comparison of the same opposite models: liberal democracy/capitalism vs communism/planned economy.	5	2026-06-05 18:37:26.712388+00
\.


ALTER TABLE public.blocks ENABLE TRIGGER ALL;

--
-- Data for Name: concept_maps; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.concept_maps DISABLE TRIGGER ALL;

COPY public.concept_maps (id, subject_id, block_id, title, source, created_at) FROM stdin;
5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	Carte Conceptuelle de la Guerre Froide	ai	2026-06-05 18:40:25.181828+00
\.


ALTER TABLE public.concept_maps ENABLE TRIGGER ALL;

--
-- Data for Name: concept_map_nodes; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.concept_map_nodes DISABLE TRIGGER ALL;

COPY public.concept_map_nodes (id, map_id, label, parent_id) FROM stdin;
d05bedd4-5008-4d48-bf01-e33e2dc6b276	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	La Guerre Froide (1947-1991)	\N
ab56f097-347c-4afd-8689-cfb8da9d58b5	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Bipolarisation du monde	d05bedd4-5008-4d48-bf01-e33e2dc6b276
28e58546-3e97-4b00-ac3c-c7135b279b84	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Bloc Occidental (USA)	ab56f097-347c-4afd-8689-cfb8da9d58b5
effdf298-ac8a-4dba-a57b-cba5cf88d221	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Bloc Oriental (URSS)	ab56f097-347c-4afd-8689-cfb8da9d58b5
70430fc9-f71f-4e21-b70d-e959e84157bb	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Doctrine du Containment	28e58546-3e97-4b00-ac3c-c7135b279b84
06a3d1b2-bb56-4ca7-9ebe-0377d118d63f	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Doctrine des deux camps	effdf298-ac8a-4dba-a57b-cba5cf88d221
6905d99d-8c48-4339-8fbf-aedcf25e90ad	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Dissuasion Nucléaire	d05bedd4-5008-4d48-bf01-e33e2dc6b276
981e40ce-9f34-4573-abfa-5bcfad917ade	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Crises Majeures	d05bedd4-5008-4d48-bf01-e33e2dc6b276
f4a502d1-0656-4b79-badc-79e9f663b00f	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Guerres par procuration	981e40ce-9f34-4573-abfa-5bcfad917ade
d5f1349b-6142-4b05-9a75-ab50523f0afa	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Détente et Coexistence	d05bedd4-5008-4d48-bf01-e33e2dc6b276
c35b80d4-15d4-4f0b-b912-185bf39f9a7a	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Réformes de Gorbatchev	d05bedd4-5008-4d48-bf01-e33e2dc6b276
d75d9567-65fb-48cb-aead-3db53cbbf262	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	Effondrement du Bloc de l'Est	d05bedd4-5008-4d48-bf01-e33e2dc6b276
\.


ALTER TABLE public.concept_map_nodes ENABLE TRIGGER ALL;

--
-- Data for Name: concept_map_edges; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.concept_map_edges DISABLE TRIGGER ALL;

COPY public.concept_map_edges (id, map_id, from_node, to_node, label) FROM stdin;
c7752a67-008b-483e-b2ad-a0fb10d7373d	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	28e58546-3e97-4b00-ac3c-c7135b279b84	70430fc9-f71f-4e21-b70d-e959e84157bb	met en œuvre
511e8f5c-eb16-4ba4-bf53-94374ac904ce	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	70430fc9-f71f-4e21-b70d-e959e84157bb	06a3d1b2-bb56-4ca7-9ebe-0377d118d63f	s'oppose à
cddff7ef-0f2c-4107-8aea-38e210d35fe7	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	6905d99d-8c48-4339-8fbf-aedcf25e90ad	981e40ce-9f34-4573-abfa-5bcfad917ade	évite l'affrontement direct
1554c78a-b53c-486d-a42e-673abd301d3a	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	981e40ce-9f34-4573-abfa-5bcfad917ade	f4a502d1-0656-4b79-badc-79e9f663b00f	inclut
d7469c64-f6ce-432b-b089-fcb57d415fdd	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	d5f1349b-6142-4b05-9a75-ab50523f0afa	c35b80d4-15d4-4f0b-b912-185bf39f9a7a	précède
0505330d-7a63-4722-9715-628067071158	5e1ec2ea-ef8f-4085-bcdb-2b6f95f1c1e6	c35b80d4-15d4-4f0b-b912-185bf39f9a7a	d75d9567-65fb-48cb-aead-3db53cbbf262	entraîne
\.


ALTER TABLE public.concept_map_edges ENABLE TRIGGER ALL;

--
-- Data for Name: cornell_notes; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.cornell_notes DISABLE TRIGGER ALL;

COPY public.cornell_notes (id, subject_id, block_id, title, body, summary, created_at) FROM stdin;
06c468c1-823c-4647-a165-32ba10c585bf	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	La Guerre Froide (1947-1991)	I. Origines et Formation des Blocs (1945-1949)\n- Rupture alliance Alliés (Yalta/Potsdam) $\\rightarrow$ opposition Capitalisme/Démocratie (USA) vs Communisme/Économie planifiée (URSS).\n- 1947: Doctrine Truman (containment/endiguement) $\\rightarrow$ Plan Marshall (aide écon. pour limiter influence soviétique).\n- Réponse URSS: Doctrine Jdanov (2 camps: impérialiste vs anti-impérialiste).\n- Crise de Berlin (1948-1949): Blocus de Staline $\\rightarrow$ Pont aérien $\\rightarrow$ Division Allemagne (RFA Ouest, RDA Est).\n- Alliances militaires: OTAN (1949, USA) vs Pacte de Varsovie (1955, URSS).\n- 1949: URSS obtient l'arme atomique + Chine devient communiste.\n\nII. Grandes Crises et Tensions (1950-1962)\n- Guerre de Corée (1950-1953): 1er conflit armé majeur, 'guerre par procuration' $\\rightarrow$ Armistice 38e parallèle.\n- Course aux armements: 'Équilibre de la terreur' / Dissuasion nucléaire (Destruction Mutuelle Assurée).\n- Conquête spatiale: Spoutnik (1957, URSS) vs Homme sur la Lune (1969, USA).\n- Mur de Berlin (août 1961): Symbole du 'rideau de fer' pour stopper fuites RDA $\\rightarrow$ Ouest.\n- Crise des missiles de Cuba (1962): Moment le plus dangereux $\\rightarrow$ Blocus naval Kennedy $\\rightarrow$ Retrait missiles Cuba/Turquie $\\rightarrow$ Installation 'téléphone rouge'.\n\nIII. Coexistence Pacifique et Détente (1962-1975)\n- Coexistence pacifique (Khrouchtchev): Concurrence idéologique/écon. plutôt que militaire.\n- Détente: Négociations SALT (limitation armements nucléaires) et Ostpolitik (Willy Brandt).\n- Accords d'Helsinki (1975): Reconnaissance frontières et respect droits de l'homme.\n- Fragilité: Guerre du Vietnam (jusqu'à 1975), USA s'enlisent.\n\nIV. Fin de la Guerre Froide (1975-1991)\n- 'Guerre fraîche' (fin 70s): Invasion Afghanistan (1979) + Reagan (IDS/Guerre des étoiles).\n- 1985: Gorbatchev $\\rightarrow$ Perestroïka (écon) et Glasnost (politique).\n- 1989: Chute du Mur de Berlin (9 nov) $\\rightarrow$ Effondrement bloc de l'Est.\n- 1991: Dislocation de l'URSS (25 déc) $\\rightarrow$ Monde unipolaire (USA seule superpuissance).	La guerre froide a été une bipolarisation mondiale entre les USA et les USA et l'URSS, évitant l'affrontement direct grâce à la dissuasion nucléaire. Après des crises majeures (Berlin, Cuba, Corée), le monde a alterné entre tensions et détente. Elle s'est achevée en 1991 avec l'effondrement économique et politique de l'URSS.	2026-06-05 18:41:08.334186+00
\.


ALTER TABLE public.cornell_notes ENABLE TRIGGER ALL;

--
-- Data for Name: flashcards; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.flashcards DISABLE TRIGGER ALL;

COPY public.flashcards (id, subject_id, block_id, front, back, hint, source, cornell_note_id, stability, difficulty, state, due, last_reviewed, reps, lapses, created_at) FROM stdin;
53dcefe2-76a5-4af0-8895-5f51083d5074	514807e5-57aa-4a7e-a3cc-1f8b725a4051	11d63919-6e8d-4a92-bbf2-f521964a5b44	Quelle est la période exacte de la guerre froide ?	1947-1991	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
ba6f469d-5da6-4077-b539-37e1c743181a	514807e5-57aa-4a7e-a3cc-1f8b725a4051	11d63919-6e8d-4a92-bbf2-f521964a5b44	Pourquoi la guerre froide est-elle qualifiée de « froide » ?	Parce que les deux superpuissances ne s'affrontent jamais directement sur un champ de bataille par crainte d'une guerre nucléaire.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
feaa8174-17c4-4234-a116-9454f1a842d6	514807e5-57aa-4a7e-a3cc-1f8b725a4051	11d63919-6e8d-4a92-bbf2-f521964a5b44	Qui a popularisé l'expression « guerre froide » en 1947 ?	Walter Lippmann	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
2220c4ee-0eab-45c5-b2f6-afb85c311e19	514807e5-57aa-4a7e-a3cc-1f8b725a4051	11d63919-6e8d-4a92-bbf2-f521964a5b44	Comment Churchill nomme-t-il la séparation du monde en deux blocs dès 1946 ?	Le « rideau de fer »	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
b94177e0-9b46-4a11-b35a-6e2455cd56ec	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Quelles conférences de 1945 révèlent les désaccords sur l'avenir de l'Europe ?	Yalta et Potsdam	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
9dc18e9b-db94-4fc0-aa2d-c4fd8a4809d9	514807e5-57aa-4a7e-a3cc-1f8b725a4051	0ef6a2e9-0f80-4ef8-a681-5802e94f0f86	Quels sont les deux modèles opposés durant la guerre froide ?	Démocratie libérale/capitalisme (USA) vs Communisme/économie planifiée (URSS)	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
6a0f6bbd-13ef-4b92-b287-3b137cb4b2ca	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Qu'est-ce que la doctrine du « containment » (endiguement) de Truman en 1947 ?	L'idée que les États-Unis aideront tout peuple libre menacé par le communisme.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
4e20650b-0a62-4420-86ad-3e2963db1758	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Quel est l'objectif du plan Marshall (1947) ?	Reconstruire l'Europe économiquement pour l'éloigner de l'influence soviétique.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
e7d05701-5934-49ca-9963-ea64101cb663	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Quelle est la doctrine soviétique formulée par Andreï Jdanov en 1947 ?	La doctrine des « deux camps » (camp impérialiste vs camp anti-impérialiste).	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
5aa5775d-c7c4-4d60-9366-a791b339c17d	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Pourquoi Staline a-t-il ordonné le blocus de Berlin-Ouest en juin 1948 ?	Pour chasser les Occidentaux de la ville.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
b6ecf933-6ebc-4220-a4d3-02f3fd49c917	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Comment les Américains et Britanniques ont-ils réagi au blocus de Berlin (1948-1949) ?	Par un pont aérien pour ravitailler la ville.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
a097461f-94a7-4ebe-87d1-793d17ac698e	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	Quelle est la conséquence politique de la crise de Berlin en 1949 ?	La division de l'Allemagne en deux États : la RFA (Ouest) et la RDA (Est).	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
032e9e8f-e06d-4c2e-af5d-26ab21d858ae	514807e5-57aa-4a7e-a3cc-1f8b725a4051	0ef6a2e9-0f80-4ef8-a681-5802e94f0f86	Qu'est-ce que l'OTAN et quand a-t-elle été créée ?	Organisation du traité de l'Atlantique Nord, créée en 1949 autour des États-Unis.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
2c410563-1b6e-429a-a0c6-c54568937cb5	514807e5-57aa-4a7e-a3cc-1f8b725a4051	0ef6a2e9-0f80-4ef8-a681-5802e94f0f86	Qu'est-ce que le pacte de Varsovie et quand a-t-il été créé ?	Alliance militaire soviétique créée en 1955 autour de l'URSS.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
59032f05-1432-4d4a-9876-00fef67d125d	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Quels deux événements de 1949 renforcent le camp communiste ?	L'obtention de l'arme atomique par l'URSS et la victoire de Mao Zedong en Chine.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
14d0cbaa-cc67-425e-a51d-ac10490ae110	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Qu'est-ce que la guerre de Corée (1950-1953) ?	Un conflit où le Nord communiste a envahi le Sud, se terminant par un armistice au 38e parallèle.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
396e13ef-0ac0-45ea-8160-b42a727c0aa0	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Pourquoi la guerre de Corée est-elle un exemple de « guerre par procuration » ?	Parce que les superpuissances s'affrontent indirectement par alliés interposés.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
5a023048-c939-4efd-88f5-649dfb577da2	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Sur quoi repose l'« équilibre de la terreur » ?	Sur la doctrine de la destruction mutuelle assurée (dissuasion nucléaire).	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
7afe12aa-8729-44e5-a870-5cd5a38b0a91	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Quel satellite a été lancé par l'URSS en 1957 ?	Spoutnik	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
9b0009f2-7791-4216-a7f3-7f562a7692c3	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Quand les États-Unis ont-ils envoyé le premier homme sur la Lune ?	1969	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
95f0f49b-cedf-4694-927e-7a77b6da3cb5	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Pourquoi la RDA a-t-elle érigé le mur de Berlin en août 1961 ?	Pour stopper la fuite des Allemands de l'Est vers l'Ouest.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
8f92026e-e333-4261-aeab-9fbcf14e1b7e	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Quelle est la symbolique du mur de Berlin ?	Le symbole le plus visible de la division du monde et du « rideau de fer ».	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
528a53c0-28ea-4b4f-9ec3-3cd3921963d0	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Quelle est la cause de la crise des missiles de Cuba en octobre 1962 ?	L'installation secrète de missiles soviétiques à Cuba, à portée des USA.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
10d32b3f-f563-428c-bf3c-3d3ece30f126	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Comment s'est résolue la crise des missiles de Cuba ?	Khrouchtchev retire les missiles de Cuba contre le retrait discret des missiles américains de Turquie.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
15f02727-642f-4700-9595-a9763878c10d	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Quelle mesure de communication a été installée après la crise de Cuba ?	Le « téléphone rouge » reliant Washington et Moscou.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
5220ce9a-e231-446a-8200-b2d2222624e5	514807e5-57aa-4a7e-a3cc-1f8b725a4051	a03287a6-426b-4eb3-b5d6-43b332f95fad	Qu'est-ce que la « coexistence pacifique » prônée par Khrouchtchev dès 1956 ?	L'idée que les deux systèmes peuvent coexister et se concurrencer sans conflit militaire.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
eec14ff6-a1fd-4a34-a17c-3ec98441ca10	514807e5-57aa-4a7e-a3cc-1f8b725a4051	a03287a6-426b-4eb3-b5d6-43b332f95fad	Que sont les accords SALT (à partir de 1972) ?	Accords de limitation des armements stratégiques pour plafonner les arsenaux nucléaires.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
fe26b469-9d5c-4f0a-be70-138677cbbf83	514807e5-57aa-4a7e-a3cc-1f8b725a4051	a03287a6-426b-4eb3-b5d6-43b332f95fad	Qu'est-ce que l'Ostpolitik de Willy Brandt ?	Une politique d'ouverture à l'Est pour améliorer les relations entre les deux Allemagnes.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
e087d3be-ecf1-4cee-8332-f3e3712097a1	514807e5-57aa-4a7e-a3cc-1f8b725a4051	a03287a6-426b-4eb3-b5d6-43b332f95fad	Que reconnaissent les accords d'Helsinki de 1975 ?	Les frontières issues de la guerre et le respect des droits de l'homme.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
56bba97b-85cc-4278-8553-138a25499cc9	514807e5-57aa-4a7e-a3cc-1f8b725a4051	a03287a6-426b-4eb3-b5d6-43b332f95fad	Quel conflit indirect a duré jusqu'en 1975 et a enlisé les États-Unis ?	La guerre du Vietnam	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
1583f8b2-4f8a-4809-bdee-3ef49fbca4a7	514807e5-57aa-4a7e-a3cc-1f8b725a4051	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	Qu'est-ce que le programme IDS (guerre des étoiles) de Ronald Reagan ?	Un programme de défense technologique pour contrer l'URSS, que l'économie soviétique ne pouvait suivre.	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
5fbaca70-7be3-4c42-bea8-5ec57aac95bd	514807e5-57aa-4a7e-a3cc-1f8b725a4051	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	Quelles sont les deux réformes de Gorbatchev à partir de 1985 ?	La perestroïka (économie) et la glasnost (transparence politique).	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
f5e55cd2-eea4-4787-8c9d-02bccc6a8315	514807e5-57aa-4a7e-a3cc-1f8b725a4051	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	Quelle date marque la chute du mur de Berlin ?	9 novembre 1989	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
e0c446b4-ebb6-473b-91ad-ce66c6a99f8f	514807e5-57aa-4a7e-a3cc-1f8b725a4051	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	Quand l'URSS a-t-elle officiellement disparu ?	25 décembre 1991	\N	ai	\N	\N	\N	new	2026-06-05 18:38:36.822771+00	\N	0	0	2026-06-05 18:38:36.822771+00
\.


ALTER TABLE public.flashcards ENABLE TRIGGER ALL;

--
-- Data for Name: cornell_cues; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.cornell_cues DISABLE TRIGGER ALL;

COPY public.cornell_cues (id, note_id, question, answer, flashcard_id) FROM stdin;
34e763a3-dd86-43ca-9e07-97547bc9138f	06c468c1-823c-4647-a165-32ba10c585bf	Qu'est-ce que la guerre froide ?	Un affrontement indirect entre USA et URSS (1947-1991) marqué par une opposition idéologique, économique et militaire sans combat direct.	\N
b0d91657-0425-482a-89d1-0da31cb62f18	06c468c1-823c-4647-a165-32ba10c585bf	Quelle est la doctrine Truman et son outil économique ?	Le containment (endiguement) visant à stopper l'expansion du communisme, concrétisé par le plan Marshall.	\N
32f78591-0480-4eff-9340-dee24368b445	06c468c1-823c-4647-a165-32ba10c585bf	Comment s'est terminée la crise du blocus de Berlin (1948-1949) ?	Par un pont aérien allié et la division de l'Allemagne en deux États : la RFA et la RDA.	\N
33dab042-dd9b-48d1-a558-f19a8d1136b6	06c468c1-823c-4647-a165-32ba10c585bf	Quelles sont les deux alliances militaires opposées ?	L'OTAN (1949, bloc Ouest) et le Pacte de Varsovie (1955, bloc Est).	\N
f3635f84-b069-460a-96b9-2424d999c54f	06c468c1-823c-4647-a165-32ba10c585bf	Qu'est-ce qu'une 'guerre par procuration' ?	Un conflit où les superpuissances s'affrontent indirectement via des alliés, comme lors de la guerre de Corée.	\N
3b3b82ea-769a-4505-a710-6f3c81c3064e	06c468c1-823c-4647-a165-32ba10c585bf	Qu'est-ce que l'équilibre de la terreur ?	Une situation où les deux camps possèdent l'arme nucléaire, rendant toute attaque mutuellement destructrice (dissuasion).	\N
ee63af73-fac1-40b9-8463-1b5a01160cb9	06c468c1-823c-4647-a165-32ba10c585bf	Pourquoi le mur de Berlin a-t-il été construit en 1961 ?	Pour empêcher les habitants de la RDA (Est) de fuir vers Berlin-Ouest.	\N
3491ad10-0913-403e-8304-d8bfe9eba31a	06c468c1-823c-4647-a165-32ba10c585bf	Quel événement marque le tournant vers la détente après la crise de Cuba (1962) ?	La prise de conscience du risque nucléaire, menant à la création du 'téléphone rouge' et à la coexistence pacifique.	\N
0a9a97dd-19c2-4351-ae00-b4f9edf4473b	06c468c1-823c-4647-a165-32ba10c585bf	Quelles étaient les réformes de Gorbatchev ?	La perestroïka (restructuration économique) et la glasnost (transparence politique).	\N
770e46f3-07bc-4dde-9ff1-15c28abcac79	06c468c1-823c-4647-a165-32ba10c585bf	Quel événement symbolise la fin de la guerre froide ?	La chute du mur de Berlin (1989) et la dislocation finale de l'URSS en 1991.	\N
\.


ALTER TABLE public.cornell_cues ENABLE TRIGGER ALL;

--
-- Data for Name: exams; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.exams DISABLE TRIGGER ALL;

COPY public.exams (id, subject_id, title, time_limit_s, source, created_at) FROM stdin;
fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	514807e5-57aa-4a7e-a3cc-1f8b725a4051	Examen blanc — Guerre froide	2700	ai	2026-06-05 18:39:25.2084+00
\.


ALTER TABLE public.exams ENABLE TRIGGER ALL;

--
-- Data for Name: feynman_concepts; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.feynman_concepts DISABLE TRIGGER ALL;

COPY public.feynman_concepts (id, subject_id, block_id, title, hint, source, created_at) FROM stdin;
dc433269-b519-4914-9d86-9a33031c4828	514807e5-57aa-4a7e-a3cc-1f8b725a4051	11d63919-6e8d-4a92-bbf2-f521964a5b44	Pourquoi la guerre est-elle dite « froide » ?	Expliquer que l'antagonisme est total (idéologique, militaire, culturel) mais qu'il n'y a pas d'affrontement direct entre les USA et l'URSS par peur d'une guerre nucléaire mutuellement destructrice.	ai	2026-06-05 18:39:54.470941+00
7a38b2e1-56e3-4589-8a2a-cab3019331cf	514807e5-57aa-4a7e-a3cc-1f8b725a4051	55b1a5e7-52bf-403e-8c1d-fab628127618	Comment le Plan Marshall a-t-il servi à lutter contre le communisme ?	Expliquer l'idée d'endiguement (containment) : donner de l'argent pour reconstruire l'Europe et rendre les gens heureux et prospères pour qu'ils ne soient pas attirés par lecommunisme.	ai	2026-06-05 18:39:54.470941+00
4c544b95-d6a6-40ff-b1d5-b00b6903f237	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Qu'est-ce qu'une « guerre par procuration » ?	Expliquer que les superpuissances ne se battent pas elles-mêmes, mais soutiennent des alliés dans des conflits locaux (comme en Corée ou au Vietnam) pour gagner de l'influence sans risquer l'apocalypse nucléaire.	ai	2026-06-05 18:39:54.470941+00
21c433d8-91fb-4b47-97bf-f94fb5ea04d5	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Pourquoi l'« équilibre de la terreur » fonctionne-t-il ?	Expliquer la notion de dissuasion nucléaire : si l'un attaque, l'autre peut répondre et détruire tout le monde. C'est donc l'un des paradoxes où la peur empêche la guerre.	ai	2026-06-05 18:39:54.470941+00
aa0fade9-eaaa-4c6b-87c1-1919626e48ae	514807e5-57aa-4a7e-a3cc-1f8b725a4051	5700703a-5246-4cf1-b66c-b98343b666e3	Pourquoi la crise des missiles de Cuba a-t-elle été un tournant ?	Expliquer que c'est le moment où le monde a frôlé la guerre nucléaire, ce qui a conduit à la création du « téléphone rouge » pour mieux communiquer et éviter l'escalade.	ai	2026-06-05 18:39:54.470941+00
a287a87a-d880-4d2f-b48b-6a20392d9f6b	514807e5-57aa-4a7e-a3cc-1f8b725a4051	a03287a6-426b-4eb3-b5d6-43b332f95fad	Qu'est-ce que la « coexistence pacifique » ?	hint": "Expliquer que les deux systèmes peuvent se concurrencer sans se battre militairement, en se battant plutôt sur le terrain de la technologie, de l'économie et de l'économie.	ai	2026-06-05 18:39:54.470941+00
a49102d1-fb41-4c58-8604-2f1db0411612	514807e5-57aa-4a7e-a3cc-1f8b725a4051	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	Pourquoi l'URSS s'est-elle effondrée ?	Expliquer le rôle des problèmes économiques (économie à bout de souffle) et les réformes de Gorbatchev (perestroïka et glasnost) qui ont libéré la parole et précipité la chute des régimes communistes.	ai	2026-06-05 18:39:54.470941+00
\.


ALTER TABLE public.feynman_concepts ENABLE TRIGGER ALL;

--
-- Data for Name: questions; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.questions DISABLE TRIGGER ALL;

COPY public.questions (id, exam_id, block_id, "position", qtype, prompt, options, answer_key, explanation, points, ai_graded) FROM stdin;
48433793-35dd-46be-93d5-ab5e4f88eb7d	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	\N	0	mcq	Qui a popularisé l'expression « guerre froide » en 1947 ?	[{"key": "a", "text": "Winston Churchill"}, {"key": "b", "text": "Harry Truman"}, {"key": "c", "text": "Walter Lippmann"}, {"key": "d", "text": "Nikita Khrouchtchev"}]	\N	\N	1	f
fad93dfe-7cb9-45ea-887c-2759a3cc5958	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	55b1a5e7-52bf-403e-8c1d-fab628127618	1	true_false	Le plan Marshall était une aide économique proposée par l'URSS pour reconstruire l'Europe.	\N	false	\N	1	f
24cf9c9f-e7e1-4f0b-b0f2-9dc850300c28	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	55b1a5e7-52bf-403e-8c1d-fab628127618	2	short_answer	Comment appelle-t-on la stratégie américaine consistant à aider tout peuple libre menacé par le communisme ?	\N	\N	L'élève doit mentionner le terme 'containment' ou 'endiguement'.	2	t
b55843bb-a4e0-4007-ab15-660fcb4d3415	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	11d63919-6e8d-4a92-bbf2-f521964a5b44	3	open_ended	Expliquez pourquoi la guerre froide est qualifiée de « froide » malgré l'existence de nombreux conflits périphériques.	\N	\N	Points attendus : 1. Antagonisme total (idéologique, économique, militaire). 2. Absence d'affrontement direct entre USA et URSS. 3. Peur d'une guerre nucléaire mutuellement destructrice (dissuasion).	5	t
9d8764c2-9266-4c77-9051-b6df68abfaf6	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	\N	4	mcq	En quelle année le mur de Berlin a-t-il été érigé ?	[{"key": "a", "text": "1948"}, {"key": "b", "text": "1955"}, {"key": "c", "text": "1961"}, {"key": "d", "text": "1969"}]	\N	\N	1	f
f65a1cfd-cdd4-4f16-a735-f7ead20507fa	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	5700703a-5246-4cf1-b66c-b98343b666e3	5	true_false	La guerre de Corée s'est terminée par une victoire totale des États-Unis.	\N	false	\N	1	f
d229f005-9c18-4cac-98ff-78a87da20d5b	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	11d63919-6e8d-4a92-bbf2-f521964a5b44	6	short_answer	Quel événement symbolise la division du monde en deux blocs rivaux selon Winston Churchill en 1946 ?	\N	\N	Le 'rideau de fer'.	2	t
f35319c7-63c4-4cd4-b1a4-06fccb8974c2	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	\N	7	mcq	Quel était l'objectif principal des accords SALT signés à partir de 1972 ?	[{"key": "a", "text": "L'abolition complète des armes nucléaires"}, {"key": "b", "text": "Le plafonnement des arsenaux nucléaires"}, {"key": "c", "text": "La création de l'OTAN"}, {"key": "d", "text": "La signature d'un traité de paix en Corée"}, {"key": "e", "text": "La fin de la guerre du Vietnam"}]	\N	\N	1	f
a797acf0-a670-40f0-b5ae-8c22a4b85bce	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	5700703a-5246-4cf1-b66c-b98343b666e3	8	open_ended	Analysez la crise des missiles de Cuba en 1962 : causes, déroulement et conséquences.	\N	\N	Causes : Installation de missiles soviétiques à Cuba. Déroulement : Blocus naval de Kennedy, tension extrême. Conséquences : Retrait des missiles de Cuba et de Turquie, installation du téléphone rouge, début de la détente.	5	t
772ff88d-152f-4711-83cd-3ebdde5da5f1	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	0ef6a2e9-0f80-4ef8-a681-5802e94f0f86	9	true_false	Le pacte de Varsovie a été créé en 1955 autour de l'URSS.	\N	true	\N	1	f
0d966254-9d18-4524-99d3-395580fc0079	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	10	short_answer	Quelles sont les deux réformes majeures engagées par Mikhaïl Gorbatchev à partir de 1985 ?	\N	\N	La perestroïka (restructuration économique) et la glasnost (transparence politique).	3	t
d6623940-4c8f-41df-ba63-79dc4320960b	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	\N	11	mcq	Quelle était la doctrine soviétique formulée par Andreï Jdanov ?	[{"key": "a", "text": "Le containment"}, {"key": "b", "text": "La coexistence pacifique"}, {"key": "c", "text": "La doctrine des « deux camps »"}, {"key": "d", "text": "La doctrine de la destruction mutuelle assurée"}, {"key": "e", "text": "La doctrine de l'Ostpolitik"}, {"key": "f", "text": "La doctrine de l'unipolarité"}]	\N	\N	1	f
f4cfcc78-5971-4d43-b483-4c0ae4ad3401	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	12	true_false	L'URSS a envahi l'Afghanistan en 1979, relançant ainsi les tensions.	\N	true	\N	1	f
613c6845-9d2f-4e73-a732-5d5d0e83eefd	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	5700703a-5246-4cf1-b66c-b98343b666e3	13	short_answer	Qu'est-ce que la « dissuasion nucléaire » ou l'« équilibre de la terreur » ?	\N	\N	L'idée que chaque camp possède un arsenal capable de détruire l'adversaire, rendant un affrontement direct impossible car mutuellement destructeur.	3	t
e44e6abb-ce9d-4eff-8550-b17b433d0b0c	fb72aabf-0e25-432c-a5fb-c0df7f7f6b0e	f0c24abb-9d1f-4671-a050-ef2fb71c2ca0	14	open_ended	Décrivez le processus de fin de la guerre froide entre 1989 et 1991.	\N	\N	Points attendus : 1. Chute du mur de Berlin (9 nov 1989). 2. Chute des régimes communistes en Europe de l'Est. 3. Réunification de l'Allemagne (1990). 4. Dislocation officielle de l'URSS (25 déc 1991).	5	t
\.


ALTER TABLE public.questions ENABLE TRIGGER ALL;

--
-- Data for Name: schema_assets; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.schema_assets DISABLE TRIGGER ALL;

COPY public.schema_assets (id, subject_id, block_id, title, reference, drawing, created_at, updated_at) FROM stdin;
a0d7339d-60b4-4930-bf10-e511c72721d6	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	L'opposition des deux blocs (1947-1949)	Un schéma conceptuel opposant deux colonnes : Bloc Ouest (USA, démocratie libérale, capitalisme, Doctrine Truman/Containment, Plan Marshall, OTAN) face au Bloc Est (URSS, communisme, économie planifiée, Doctrine Jdanov, Pacte de Varsovie). Une flèche centrale symbolisant le 'Rideau de fer' séparant les deux.	\N	2026-06-05 18:45:07.500071+00	2026-06-05 18:45:07.500071+00
24397a98-fc1e-48d2-92ab-a53deca32f65	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	Le mécanisme de la dissuasion nucléaire	Un schéma de cause à effet : Arsenaux nucléaires massifs $\\rightarrow$ Capacité de destruction mutuelle assurée $\\rightarrow$ 'Équilibre de la terreur' $\\rightarrow$ Absence d'affrontement direct entre superpuissances.	\N	2026-06-05 18:45:07.500071+00	2026-06-05 18:45:07.500071+00
8677e23f-b3e4-4261-8203-4b22ea4b548e	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	La chronologie des relations Est-Ouest	Une frise chronologique ou un graphique de tension (courbe de température) avec les points clés : 1947-1949 (Rupture et Blocus de Berlin), 1950-1962 (Tensions et Crise des missiles de Cuba), 1962-1975 (Détente et Accords SALT/Helsinki), 1975-1989 (Guerre fraîche et chute du Mur), 1991 (Dislocation de l'URSS).	\N	2026-06-05 18:45:07.500071+00	2026-06-05 18:45:07.500071+00
\.


ALTER TABLE public.schema_assets ENABLE TRIGGER ALL;

--
-- Data for Name: source_documents; Type: TABLE DATA; Schema: public; Owner: -
--

ALTER TABLE public.source_documents DISABLE TRIGGER ALL;

COPY public.source_documents (id, subject_id, block_id, title, content, created_at) FROM stdin;
5a323b17-6f8a-4360-8857-385cb2d3caed	514807e5-57aa-4a7e-a3cc-1f8b725a4051	\N	Cours — La Guerre froide (1947-1991)	LA GUERRE FROIDE (1947-1991)\n\nINTRODUCTION\nLa guerre froide désigne la période d'affrontement indirect entre les États-Unis et l'Union soviétique (URSS) de 1947 à 1991. C'est une « guerre » car l'antagonisme est total — idéologique, économique, militaire, culturel — mais elle est « froide » car les deux superpuissances ne s'affrontent jamais directement sur un champ de bataille, par crainte d'une guerre nucléaire qui serait mutuellement destructrice. L'expression est popularisée par le journaliste Walter Lippmann en 1947. Le monde se structure alors en deux blocs rivaux séparés par ce que Churchill nomme dès 1946 un « rideau de fer ».\n\nI. LES ORIGINES ET LA FORMATION DES BLOCS (1945-1949)\n\nDès la fin de la Seconde Guerre mondiale, l'alliance entre les Alliés se fissure. Les conférences de Yalta (février 1945) et de Potsdam (juillet-août 1945) révèlent des désaccords profonds sur l'avenir de l'Europe, notamment de l'Allemagne et de la Pologne. Deux modèles s'opposent radicalement : d'un côté, la démocratie libérale et le capitalisme américains ; de l'autre, le communisme et l'économie planifiée soviétiques.\n\nEn 1947, la rupture est consommée. Le président américain Harry Truman énonce la doctrine du « containment » (endiguement) : les États-Unis aideront tout peuple libre menacé par le communisme. Cette doctrine se concrétise par le plan Marshall, une aide économique massive proposée à l'Europe pour la reconstruire et l'éloigner de l'influence soviétique. En réponse, Andreï Jdanov formule la doctrine soviétique des « deux camps » : le camp impérialiste mené par Washington et le camp anti-impérialiste mené par Moscou. L'URSS refuse le plan Marshall et l'impose à ses satellites.\n\nLa première grande crise éclate à Berlin. La ville, située en zone soviétique, est divisée en quatre secteurs. En juin 1948, Staline ordonne le blocus de Berlin-Ouest pour en chasser les Occidentaux. Les Américains et les Britanniques répliquent par un gigantesque pont aérien qui ravitaille la ville pendant près d'un an. Staline lève le blocus en mai 1949 : c'est un échec soviétique. Cette crise scelle la division de l'Allemagne en deux États en 1949 : la RFA (République fédérale d'Allemagne) à l'Ouest et la RDA (République démocratique allemande) à l'Est.\n\nLes blocs se dotent d'alliances militaires : l'OTAN (Organisation du traité de l'Atlantique Nord) est créée en 1949 autour des États-Unis ; le pacte de Varsovie sera créé en 1955 autour de l'URSS. En 1949, deux événements majeurs renforcent le camp communiste : l'URSS obtient l'arme atomique, mettant fin au monopole nucléaire américain, et la Chine de Mao Zedong devient communiste.\n\nII. LES GRANDES CRISES ET LE TEMPS DES TENSIONS (1950-1962)\n\nLa guerre de Corée (1950-1953) est le premier conflit armé majeur de la guerre froide. La Corée du Nord communiste envahit la Corée du Sud. L'ONU, sous direction américaine, intervient au Sud ; la Chine soutient le Nord. La guerre se solde par un armistice en 1953 qui fige la frontière au 38e parallèle, sans vainqueur. C'est l'exemple type d'une « guerre par procuration », où les superpuissances s'affrontent indirectement par alliés interposés.\n\nLa période est marquée par la course aux armements et la « dissuasion nucléaire ». Les deux camps accumulent des arsenaux capables de détruire l'adversaire : c'est l'équilibre de la terreur, fondé sur la doctrine de la destruction mutuelle assurée. La conquête spatiale devient un autre terrain de rivalité : l'URSS lance le premier satellite Spoutnik en 1957, les États-Unis enverront le premier homme sur la Lune en 1969.\n\nÀ Berlin, la tension culmine de nouveau. Pour stopper l'hémorragie des Allemands de l'Est fuyant vers l'Ouest, la RDA érige en août 1961 le mur de Berlin. Ce mur devient le symbole le plus visible de la division du monde et du « rideau de fer ».\n\nLa crise des missiles de Cuba, en octobre 1962, est le moment où le monde frôle la guerre nucléaire. Les Soviétiques installent secrètement des missiles à Cuba, à portée du territoire américain. Le président John Kennedy impose un blocus naval de l'île et exige le retrait des missiles. Après treize jours d'une tension extrême, Nikita Khrouchtchev recule et retire les missiles, en échange du retrait discret de missiles américains de Turquie. Cette crise, la plus dangereuse de la guerre froide, marque un tournant : les deux camps prennent conscience du danger et installent un « téléphone rouge » reliant directement Washington et Moscou pour éviter l'escalade.\n\nIII. LA COEXISTENCE PACIFIQUE ET LA DÉTENTE (1962-1975)\n\nAprès la peur de 1962 s'ouvre une période d'apaisement relatif. Dès 1956, Khrouchtchev avait prôné la « coexistence pacifique » : l'idée que les deux systèmes peuvent coexister et se concurrencer sur les terrains économique, technologique et idéologique plutôt que militaire. La détente des années 1960-1970 voit se multiplier les négociations.\n\nLes accords de limitation des armements stratégiques (SALT) sont signés à partir de 1972 pour plafonner les arsenaux nucléaires. En Europe, la politique d'ouverture à l'Est (l'Ostpolitik du chancelier ouest-allemand Willy Brandt) améliore les relations entre les deux Allemagnes. Les accords d'Helsinki, en 1975, reconnaissent les frontières issues de la guerre et engagent les États sur le respect des droits de l'homme.\n\nLa détente reste cependant fragile et n'empêche pas la poursuite des affrontements indirects, notamment la longue et coûteuse guerre du Vietnam (jusqu'en 1975), où les États-Unis s'enlisent face au communisme soutenu par l'URSS et la Chine.\n\nIV. LA FIN DE LA GUERRE FROIDE (1975-1991)\n\nÀ la fin des années 1970, les tensions reprennent — on parle parfois de « guerre fraîche ». L'invasion de l'Afghanistan par l'URSS en 1979 et l'arrivée au pouvoir de dirigeants fermes, comme Ronald Reagan aux États-Unis, relancent la course aux armements. Reagan qualifie l'URSS d'« empire du mal » et lance le programme de défense IDS (Initiative de défense stratégique, dite « guerre des étoiles »), une course technologique que l'économie soviétique, à bout de souffle, ne peut suivre.\n\nÀ partir de 1985, Mikhaïl Gorbatchev arrive au pouvoir en URSS et engage des réformes profondes : la perestroïka (restructuration économique) et la glasnost (transparence et libéralisation politique). Il renonce à intervenir militairement dans les pays satellites. Cette ouverture précipite l'effondrement du bloc de l'Est.\n\nEn 1989, les régimes communistes d'Europe de l'Est tombent les uns après les autres, le plus souvent pacifiquement. Le symbole de cet effondrement est la chute du mur de Berlin, le 9 novembre 1989. L'Allemagne est réunifiée en 1990. Enfin, l'URSS elle-même se disloque : elle disparaît officiellement le 25 décembre 1991. La guerre froide est terminée, et les États-Unis restent l'unique superpuissance d'un monde devenu unipolaire.\n\nCONCLUSION\nPendant plus de quarante ans, la guerre froide a structuré les relations internationales autour d'une bipolarisation Est-Ouest. Jamais l'affrontement direct n'a eu lieu entre les deux Grands, grâce à la dissuasion nucléaire, mais le monde a vécu sous la menace permanente d'une guerre totale et a connu de nombreux conflits périphériques. Sa fin, en 1991, ouvre une nouvelle ère et de nouveaux défis.\n	2026-06-05 18:37:04.182948+00
\.


ALTER TABLE public.source_documents ENABLE TRIGGER ALL;

--
-- PostgreSQL database dump complete
--

\unrestrict kZu4nAxJIactEFwRhAFoWvqGNajslNMSkSkELVeq9itWtuutAr1DfCXtc41aUCd

