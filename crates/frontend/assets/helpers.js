const inputs = document.querySelectorAll('.numeric-input');

inputs.forEach(input => {
    input.addEventListener('input', () => {
        input.value = input.value.replace(/\D/g, '');
    });
});
