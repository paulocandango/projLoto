-- cmd
-- net start mysql80
-- net stop mysql80

CREATE DATABASE loto;
USE loto;

CREATE TABLE `lottery` (
  `id_lottery` int NOT NULL AUTO_INCREMENT,
  `lottery_name` varchar(255) NOT NULL,
  `results_url` varchar(255) NOT NULL,
  `contest_selector` varchar(255) NOT NULL,
  `numbers_selector` varchar(255) NOT NULL,
  `award_wallet` varchar(255) DEFAULT NULL,
  `is_dinamic` tinyint(1) NOT NULL DEFAULT '1',
  PRIMARY KEY (`id_lottery`)
) ENGINE=InnoDB AUTO_INCREMENT=25 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `edition` (
  `id_edition` int NOT NULL AUTO_INCREMENT,
  `id_lottery` int NOT NULL,
  `contest_selected` varchar(255) NOT NULL,
  `numbers_selected` varchar(255) NOT NULL,
  PRIMARY KEY (`id_edition`),
  KEY `fk_edition_lottery` (`id_lottery`),
  CONSTRAINT `fk_edition_lottery` FOREIGN KEY (`id_lottery`) REFERENCES `lottery` (`id_lottery`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `bet` (
  `id_bet` int NOT NULL AUTO_INCREMENT,
  `id_lottery` int NOT NULL,
  `wallet` varchar(5000) DEFAULT NULL,
  `numbers` varchar(255) NOT NULL,
  `checking_id` varchar(255) NOT NULL,
  PRIMARY KEY (`id_bet`),
  KEY `fk_lottery` (`id_lottery`),
  CONSTRAINT `fk_lottery` FOREIGN KEY (`id_lottery`) REFERENCES `lottery` (`id_lottery`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=28 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;


INSERT INTO loto.lottery (lottery_name,results_url,contest_selector,numbers_selector,award_wallet,is_dinamic) VALUES 
('Brasil - Mega Sena','http://www.megasena.com.br/resultados','.concurso-id','.numeros-sorteados',NULL,0)
,('Brasil - Loto Facil','http://www.lotofacil.com.br/resultados','#concurso-lotofacil','.lotofacil-numeros',NULL,0)
,('United States - Power Ball','http://www.powerball.com/results','.pb-concurso-id','.pb-numeros-sorteados',NULL,0)
,('China - WelFare Lottery','http://www.cwl.gov.cn/results','.cwl-concurso-id','.cwl-numeros-sorteados',NULL,0)
,('Bob Loterry','http://localhost:8080/static/example.html','#identity','#elements',NULL,1)
;


SELECT * FROM lottery;
SELECT * FROM bet;
SELECT * FROM edition;

