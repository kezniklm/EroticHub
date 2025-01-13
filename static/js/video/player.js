document.addEventListener("htmx:load", function () {
    setupVideo();
});

function setupVideo() {
    const videoElement = document.getElementById("video-player");
    if (!videoElement) {
        return;
    }
    videojs(videoElement, {
        controls: true,
        autoplay: false,
        preload: 'auto',
        responsive: true,
        fluid: true,
        aspectRatio: "16:9",
    });
}