// Procedurally-generated 3D Lily58 rendered with Three.js.
// Each physical slot (from the Rust `layout` module) becomes a keycap whose top
// face carries a canvas-texture label. Clicking a cap raises an onSelect event.

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

const CAP = 0.92; // keycap footprint in "u"
const CAP_H = 0.5; // keycap height

export class Keyboard3D {
  constructor(container) {
    this.container = container;
    this.caps = []; // { mesh, index, baseColor }
    this.onSelect = () => {};
    this.selected = -1;
    this._initScene();
    this._bindEvents();
    this._animate();
  }

  _initScene() {
    const w = this.container.clientWidth || 800;
    const h = this.container.clientHeight || 600;

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0f1117);

    this.camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 1000);
    this.camera.position.set(0, -9, 12);

    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setSize(w, h);
    this.container.appendChild(this.renderer.domElement);

    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.target.set(0, 0, 0);

    this.scene.add(new THREE.AmbientLight(0xffffff, 0.75));
    const key = new THREE.DirectionalLight(0xffffff, 1.1);
    key.position.set(4, -6, 12);
    this.scene.add(key);
    const rim = new THREE.DirectionalLight(0x6ea8fe, 0.5);
    rim.position.set(-8, 8, 6);
    this.scene.add(rim);

    this.group = new THREE.Group();
    this.scene.add(this.group);

    this.raycaster = new THREE.Raycaster();
    this.pointer = new THREE.Vector2();
  }

  _bindEvents() {
    window.addEventListener("resize", () => this._resize());
    const el = this.renderer.domElement;
    el.addEventListener("click", (e) => this._pick(e));
  }

  _resize() {
    const w = this.container.clientWidth;
    const h = this.container.clientHeight;
    if (!w || !h) return;
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(w, h);
  }

  _pick(event) {
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    this.raycaster.setFromCamera(this.pointer, this.camera);
    const meshes = this.caps.map((c) => c.mesh);
    const hit = this.raycaster.intersectObjects(meshes, false)[0];
    if (hit) {
      const idx = hit.object.userData.index;
      this.select(idx);
      this.onSelect(idx);
    }
  }

  _animate() {
    requestAnimationFrame(() => this._animate());
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }

  // Build caps from the layout (array of Slot). Center the board at origin.
  setLayout(slots) {
    for (const c of this.caps) {
      this.group.remove(c.mesh);
      c.mesh.geometry.dispose();
      c.mesh.material.forEach((m) => {
        if (m.map) m.map.dispose();
        m.dispose();
      });
    }
    this.caps = [];

    const xs = slots.map((s) => s.x);
    const ys = slots.map((s) => s.y);
    const cx = (Math.min(...xs) + Math.max(...xs)) / 2;
    const cy = (Math.min(...ys) + Math.max(...ys)) / 2;

    for (const s of slots) {
      const geo = new THREE.BoxGeometry(CAP, CAP, CAP_H);
      // materials: [ +x, -x, +y, -y, +z(top), -z ]
      const side = new THREE.MeshStandardMaterial({ color: 0x2a2f3d, roughness: 0.8 });
      const top = new THREE.MeshStandardMaterial({ roughness: 0.55 });
      const mats = [side, side.clone(), side.clone(), side.clone(), top, side.clone()];
      const mesh = new THREE.Mesh(geo, mats);
      // y flips so the number row is at the "back" (away from camera).
      mesh.position.set(s.x - cx, -(s.y - cy), s.thumb ? -0.15 : 0);
      mesh.userData.index = s.index;
      this.group.add(mesh);
      this.caps[s.index] = {
        mesh,
        index: s.index,
        baseColor: s.thumb ? 0x243044 : 0x1e222d,
      };
    }
  }

  // Paint labels for the active layer (array of {label, is_transparent}).
  setLayer(keys) {
    for (const cap of this.caps) {
      if (!cap) continue;
      const k = keys[cap.index];
      const label = k ? k.label : "";
      const dim = k ? k.is_transparent : true;
      const topMat = cap.mesh.material[4];
      if (topMat.map) topMat.map.dispose();
      topMat.map = this._labelTexture(label, dim);
      topMat.needsUpdate = true;
      cap.mesh.material.forEach((m, i) => {
        if (i !== 4) m.color.setHex(dim ? 0x161a22 : cap.baseColor);
      });
    }
    this._repaintSelection();
  }

  _labelTexture(text, dim) {
    const size = 128;
    const c = document.createElement("canvas");
    c.width = c.height = size;
    const g = c.getContext("2d");
    g.fillStyle = dim ? "#161a22" : "#20263400";
    g.clearRect(0, 0, size, size);
    g.fillStyle = dim ? "#3b4252" : "#e6e9ef";
    g.font = `bold ${text.length > 3 ? 34 : 54}px -apple-system, sans-serif`;
    g.textAlign = "center";
    g.textBaseline = "middle";
    // Wrap long labels onto two lines.
    const words = text.split(" ");
    if (words.length > 1 && text.length > 5) {
      g.font = "bold 30px sans-serif";
      g.fillText(words[0], size / 2, size / 2 - 16);
      g.fillText(words.slice(1).join(" "), size / 2, size / 2 + 18);
    } else {
      g.fillText(text, size / 2, size / 2);
    }
    const tex = new THREE.CanvasTexture(c);
    tex.anisotropy = 4;
    return tex;
  }

  select(index) {
    this.selected = index;
    this._repaintSelection();
  }

  _repaintSelection() {
    this.caps.forEach((cap) => {
      if (!cap) return;
      const isSel = cap.index === this.selected;
      cap.mesh.material.forEach((m, i) => {
        if (i === 4) return;
        m.emissive = new THREE.Color(isSel ? 0x2b5fb0 : 0x000000);
      });
      cap.mesh.position.z = (cap.mesh.userData.thumb ? -0.15 : 0) + (isSel ? 0.12 : 0);
    });
  }
}
