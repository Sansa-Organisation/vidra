// Real usage example importing from @sansavision/vidra-player
import { Project, Scene, Layer, Easing } from '@sansavision/vidra-player';

// The engine runner dynamically evaluates this function.
// Note: written in standard JS syntax so the browser can execute it instantly!
export function createDemoProject() {
    const project = new Project(1920, 1080, 60)
        .background("#09090b");

    const s1 = new Scene("intro", 3.0);
    s1.addLayers(
        new Layer("bg").solid("#09090b"),
        new Layer("text")
            .text("Powered by JS SDK & React", "Inter", 120, "#10b981")
            .position(960, 540)
            .animate("opacity", 0, 1, 1.5, Easing.EaseOut)
    );
    project.addScene(s1);

    // Try uploading an image and referencing it here like:
    // const s2 = new Scene("img", 3.0);
    // s2.addLayer(new Layer("logo").image("your-asset-id").position(960, 540));
    // project.addScene(s2);

    return project;
}
