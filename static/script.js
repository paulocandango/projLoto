const lotterySelect = document.getElementById('lottery-select');
const numberBallsContainer = document.getElementById('number-balls');
const selectedNumbersDiv = document.getElementById('selected-numbers');
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
    if (value === 'megasena') {
        generateBalls(60);
    } else if (value === 'lotofacil') {
        generateBalls(25);
    } else if (value === 'powerball') {
        generateBalls(69);
    }

    // Zera os numeros escolhidos
    selectedNumbers = [];
    selectedNumbersDiv.innerText = "";
});

// Inicializa com Powerball
generateBalls(69);
