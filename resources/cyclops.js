
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
