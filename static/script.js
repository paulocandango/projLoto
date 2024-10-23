const lotterySelect = document.getElementById('lottery');
const numberBallsContainer = document.getElementById('number-balls');
const selectedNumbersDiv = document.getElementById('selected-numbers');
const submitbutton = document.getElementById('submit-button');
const numbers = document.getElementById('numbers');
const wallet = document.getElementById('wallet');

let selectedNumbers = [];

// Gera as bolas
function generateBalls(max) {
    if(numberBallsContainer){
        numberBallsContainer.innerHTML = '';

        for (let i = 1; i <= max; i++) {
            const ball = document.createElement('div');
            ball.className = 'ball';
            ball.innerText = i;
            ball.addEventListener('click', () => toggleBallSelection(ball, i));
            numberBallsContainer.appendChild(ball);
        }
    }
}

// Alterna a seleção da bola
function toggleBallSelection(ball, number) {
    if (selectedNumbers.includes(number)) {
        selectedNumbers = selectedNumbers.filter(num => num !== number);
        ball.classList.remove('selected');
    } else {
        selectedNumbers.push(number);
        ball.classList.add('selected');
    }
    updateSelectedNumbers();
}

// Atualiza os números selecionados
function updateSelectedNumbers() {
    selectedNumbersDiv.innerText = selectedNumbers.join(' ');
    numbers.value = selectedNumbers;
}

if(lotterySelect) {
    lotterySelect.addEventListener('change', () => {
        const value = lotterySelect.value;
        if(value === ''){
            generateBalls(0);
        } else if (value === 'Brasil - Mega Sena') {
            generateBalls(60);
        } else if (value === 'Brasil - Loto Facil') {
            generateBalls(25);
        } else if (value === 'United States - Power Ball') {
            generateBalls(69);
        } else if (value === 'China - WelFare Lottery') {
            generateBalls(33);
        } else {
            generateBalls(99);
        }

        // Zera os numeros escolhidos
        selectedNumbers = [];
        updateSelectedNumbers();
    });
}

// Inicializa com Powerball
generateBalls(0);

//----------------- REGRAS DE VALIDAÇÃO DOS TIPOS DE LOTERIAS ------------------------------------
// Adiciona verificação ao clicar no botão de submit
if(submitbutton){
    submitbutton.addEventListener('click', function(event) {

        const selectedLottery = lotterySelect.value;

        if(!isSelectionValid(selectedNumbers.length)){
            alert('Escolha os numeros de acordo com as quantidades apropriadas para o seu sorteio!');
            event.preventDefault();  // Cancela o envio do formulário
            return;
        }

        if(wallet.value === ""){
            alert('Informe o número do endereço que receberá o prêmio!');
            event.preventDefault();  // Cancela o envio do formulário
            return;
        }

    });
}

function isSelectionValid(newLength) {

    const value = lotterySelect.value;

    if (value === 'Brasil - Mega Sena') {
        return newLength >= 6 && newLength <= 15;
    } else if (value === 'Brasil - Loto Facil') {
        return newLength >= 15 && newLength <= 20;
    } else if (value === 'United States - Power Ball') {
        return newLength === 6;
    } else if (value === 'China - WelFare Lottery') {
        return newLength === 7;
    } else if (value === '') {
        return false;
    }

    return true;
}