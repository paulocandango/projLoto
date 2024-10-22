create database loto;

use loto;

CREATE TABLE Lottery (
    lottery_name VARCHAR(255) NOT NULL,                -- Nome da loteria
    results_url VARCHAR(255) NOT NULL,                 -- URL publicador dos resultados
    contest_selector VARCHAR(255) NOT NULL,            -- Seletor identificador do concurso
    numbers_selector VARCHAR(255) NOT NULL             -- Seletor identificador dos elementos sorteados
);

INSERT INTO Lottery (lottery_name, results_url, contest_selector, numbers_selector)
VALUES 
('Mega Sena', 'https://www.megasena.com.br/resultados', '.concurso-id', '.numeros-sorteados'),
('Lotofácil', 'https://www.lotofacil.com.br/resultados', '#concurso-lotofacil', '.lotofacil-numeros'),
('Powerball', 'https://www.powerball.com/results', '.pb-concurso-id', '.pb-numeros-sorteados'),
('China Welfare Lottery', 'https://www.cwl.gov.cn/results', '.cwl-concurso-id', '.cwl-numeros-sorteados');

SELECT * FROM Lottery;