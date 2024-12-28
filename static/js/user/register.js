document.addEventListener("DOMContentLoaded", initializePreview);

function initializePreview() {
    const preview = document.getElementById('image-preview');
    const fileInput = document.getElementById('profile_picture');

    setDefaultPreview(preview);

    if (fileInput) {
        fileInput.addEventListener('change', (event) => updatePreview(event, preview));
    }
}

function setDefaultPreview(preview) {
    preview.src = '/static/images/anonymous_profile_picture.jpg';
    preview.style.display = 'block';
}

function updatePreview(event, preview) {
    const file = event.target.files[0];
    if (file) {
        const reader = new FileReader();
        reader.onload = (e) => {
            preview.src = e.target.result;
            preview.style.display = 'block';
        };
        reader.readAsDataURL(file);
    } else {
        setDefaultPreview(preview);
    }
}