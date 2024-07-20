const numeric_inputs = document.querySelectorAll('.numeric-input');

numeric_inputs.forEach(input => {
    input.addEventListener('input', () => {
        input.value = input.value.replace(/\D/g, '');
    });
});
