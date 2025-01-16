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


async function setupStream(url) {
    const videoElement = document.getElementById("stream-player");
    if (!videoElement) {
        return;
    }

    const player = videojs(videoElement, {
        controls: true,
        autoplay: true,
        preload: 'auto',
        responsive: true,
        fluid: true,
        aspectRatio: "16:9",
        html5: {
            nativeAudioTracks: false,
            nativeVideoTracks: false,
            hls: {
                overrideNative: true,
            }
        },
    }, function() {
        const player = this;

        player.qualityLevels();
        console.log(url);
        player.src({
            src: url,
            type: 'application/x-mpegURL',
            withCredentials: true,
        })
        player.hlsQualitySelector({ displayCurrentQuality: true} );
    });

    if (!await isPlaylistAvailable(player)) {
        loadingPlaceholder(player).catch(console.error);
    }
}

/**
 * Displays placeholder until the stream is loaded
 * @param player VideoJS player
 * @returns {Promise<void>}
 */
async function loadingPlaceholder(player) {
    const TIMEOUT_AFTER_MS = 120000;
    const loadingHeader = document.getElementById("loading-stream-header");
    const videoElement = document.getElementById("stream-player");

    videoElement.style.display = "none";
    loadingHeader.style.display = "unset";

    let intervalID = null;
    let dotsCount = 2;
    let startTime = Date.now();
    const cycle = async function() {
        if (await isPlaylistAvailable(player)) {
            videoElement.style.display = "unset";
            loadingHeader.style.display = "none";

            player.load();
            player.play();
            clearInterval(intervalID);
            return;
        } else if ((Date.now() - startTime) > TIMEOUT_AFTER_MS) {
            clearInterval(intervalID);
            return;
        }
        dotsCount = loadingDots(dotsCount);
    }

    intervalID = setInterval(cycle, 2000);
}

function loadingDots(dotsCount) {
    const MAX_DOTS = 5;
    const headerDots = document.getElementById("stream-loading-dots");

    headerDots.innerHTML = "&middot;".repeat(dotsCount);
    return dotsCount === MAX_DOTS ? 1 : ++dotsCount;
}

/**
 * Checks if HLS playlist is already available using HTTP request
 * @param player VideoJS player
 * @returns {Promise<boolean>} true if playlist is available, otherwise false
 */
async function isPlaylistAvailable(player) {
    const streamSource = player.currentSource().src;
    const response = await fetch(streamSource);
    return response.status === 200;
}