const lotterySelect = document.getElementById('lottery-select');
const numberBallsContainer = document.getElementById('number-balls');
const selectedNumbersDiv = document.getElementById('selected-numbers');
const wallet = document.getElementById('wallet');
const submitbutton = document.getElementById('submit-button');

let selectedNumbers = [];

// Gera as bolas
function generateBalls(max) {
    numberBallsContainer.innerHTML = '';
    for (let i = 1; i <= max; i++) {
        const ball = document.createElement('div');
        ball.className = 'ball';
        ball.innerText = i;
        ball.addEventListener('click', () => toggleBallSelection(ball, i));
        numberBallsContainer.appendChild(ball);
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
}

// Habilita bolas conforme a seleção da loteria
lotterySelect.addEventListener('change', () => {
    const value = lotterySelect.value;
    if(value === ''){
        generateBalls(0);
    } else if (value === 'megasena') {
        generateBalls(60);
    } else if (value === 'lotofacil') {
        generateBalls(25);
    } else if (value === 'powerball') {
        generateBalls(69);
    } else if (value === 'chinawelfare') {
        generateBalls(33);
    }

    // Zera os numeros escolhidos
    selectedNumbers = [];
    selectedNumbersDiv.innerText = "";
});

// Inicializa com Powerball
generateBalls(0);

//----------------- REGRAS DE VALIDAÇÃO DOS TIPOS DE LOTERIAS ------------------------------------
// Adiciona verificação ao clicar no botão de submit
submitbutton.addEventListener('click', function(event) {

    const selectedLottery = lotterySelect.value;

    if(!isSelectionValid(selectedNumbers.length)){
        alert('Escolha os numeros de acordo com as quantidades apropriadas para o seu sorteio!');
        event.preventDefault();  // Cancela o envio do formulário
        return;
    }

});

function isSelectionValid(newLength) {

    const value = lotterySelect.value;

    if (value === 'megasena') {
        return newLength >= 6 && newLength <= 15;
    } else if (value === 'lotofacil') {
        return newLength >= 15 && newLength <= 20;
    } else if (value === 'powerball') {
        return newLength === 6;
    } else if (value === 'chinawelfare') {
        return newLength === 7;
    } else if (value === '') {
        return false;
    }

    return true;
}