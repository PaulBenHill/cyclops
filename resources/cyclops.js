
function close_table(name) {
    const e = document.getElementById(name);

    if (e != null) {
        e.remove();
    }
};

function summary_open(name) {
    const e = document.getElementById(name);
    if (e != null) {
        if (e.hasAttribute('hidden')) {
            e.removeAttribute('hidden');
        }
    }
};

function summary_close(name) {
    const e = document.getElementById(name);
    if (e != null) {
        e.hidden = 'hidden';
    }
};

function blink(name) {
    // Get the div element by its ID 
    const blinkDiv =
        document.getElementById(name);
    console.log("blink fired");
    // An array of colors to be used for blinking 
    const colors = ['#ff0000', 'black', '#0000ff'];

    // Variable to keep track of the 
    // current color index 
    let currentColorIndex = 0;

    // Function to toggle the background 
    // color of the div 
    function blinkBackground() {
        blinkDiv.style.backgroundColor =
            colors[currentColorIndex];
        currentColorIndex =
            (currentColorIndex + 1) % colors.length;
    }

    // Start the blinking by setting an interval  
    // that calls the blinkBackground  
    // function every 1000ms (1 second) 
    const blinkingInterval =
        setInterval(blinkBackground, 1000);

    // To stop blinking after 5 seconds,  
    // use setTimeout to clear the interval 
    setTimeout(() => {
        clearInterval(blinkingInterval);
    }, 2000);
};