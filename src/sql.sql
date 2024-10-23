create database loto;

use loto;

CREATE TABLE Lottery (
    id_lottery INT AUTO_INCREMENT PRIMARY KEY,                 -- ID único e chave primária
    lottery_name VARCHAR(255) NOT NULL,                -- Nome da loteria
    results_url VARCHAR(255) NOT NULL,                 -- URL publicador dos resultados
    contest_selector VARCHAR(255) NOT NULL,            -- Seletor identificador do concurso
    numbers_selector VARCHAR(255) NOT NULL             -- Seletor identificador dos elementos sorteados
);

CREATE TABLE Edition (
    id_edition INT AUTO_INCREMENT PRIMARY KEY,   -- ID único e chave primária
    id_lottery INT NOT NULL,                     -- Chave estrangeira para Lottery
    contest_selected VARCHAR(255) NOT NULL,      -- Concurso selecionado
    numbers_selected VARCHAR(255) NOT NULL,      -- Números sorteados
    CONSTRAINT fk_edition_lottery FOREIGN KEY (id_lottery) REFERENCES Lottery(id_lottery) ON DELETE CASCADE
);

CREATE TABLE Bet (
    id_bet INT AUTO_INCREMENT PRIMARY KEY,              -- ID único e chave primária
    id_lottery INT NOT NULL,                           -- Chave estrangeira que referencia o ID da tabela Lottery
    wallet VARCHAR(255) NOT NULL,                   -- Endereço da carteira Bitcoin
    numbers VARCHAR(255) NOT NULL,                  -- Números escolhidos
    qr_code_base64 TEXT NOT NULL,                   -- QR Code em formato Base64
    qrcode VARCHAR(255) NOT NULL,                   -- Código da fatura gerada
    checking_id VARCHAR(255) NOT NULL,              -- ID de verificação do pagamento,
    CONSTRAINT fk_lottery FOREIGN KEY (id_lottery) REFERENCES Lottery(id_lottery) ON DELETE CASCADE
);



INSERT INTO Lottery (lottery_name, results_url, contest_selector, numbers_selector)
VALUES 
('Mega Sena', 'https://www.megasena.com.br/resultados', '.concurso-id', '.numeros-sorteados'),
('Lotofacil', 'https://www.lotofacil.com.br/resultados', '#concurso-lotofacil', '.lotofacil-numeros'),
('Powerball', 'https://www.powerball.com/results', '.pb-concurso-id', '.pb-numeros-sorteados'),
('Welfare Lottery', 'https://www.cwl.gov.cn/results', '.cwl-concurso-id', '.cwl-numeros-sorteados');


INSERT INTO Lottery (lottery_name, results_url, contest_selector, numbers_selector)
VALUES 
('Loteria1', 'https://www.loteria1.com.br/resultados', '.concurso-id', '.numeros-sorteados'),
('Loteria2', 'https://www.loteria2.com.br/resultados', '.concurso-id', '.numeros-sorteados'),
('Loteria3', 'https://www.loteria3.com.br/resultados', '.concurso-id', '.numeros-sorteados');



SELECT * FROM lottery;
SELECT * FROM edition;
SELECT * FROM bet;







