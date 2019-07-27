//this is a javascript function because of the vendor prefixes
export function dofullscreen() {
    var element = document.documentElement;
    //it is not supported on Safari for iPhone.
    //there is a workaround to Add to Home Screen the webapp
    if (element.webkitRequestFullscreen) {
        element.webkitRequestFullscreen();
    } else if (element.requestFullscreen) {
        element.requestFullscreen();
    } else if (element.mozRequestFullScreen) {
        element.mozRequestFullScreen();
    } else if (element.msRequestFullscreen) {
        element.msRequestFullscreen();
    }
}