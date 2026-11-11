
struct WindSource { position:vec4f, direction:vec4f, variation:vec4f, padding:vec4f }
struct Params {
  matrix: mat4x4f,
  eye: vec4f,
  clock: vec4f,
  settings: vec4f,
  counts: vec4f,
  viewport: vec4f,
  windSource: vec4f,
  windDirection: vec4f,
  physics: vec4f,
  grab: vec4f,
  anchor: vec4f,
  sourceControl:vec4f,
  sources:array<WindSource,16>,
  graphics:vec4f,
  material:vec4f,
  inverse:mat4x4f,
  env:vec4f,
  celestial:vec4f,
  groom:vec4f,
  irradiance:array<vec4f,9>,
  look:vec4f,
  illumination:vec4f,
  studio_light:vec4f,
  detail:vec4f,
}
struct Node { position: vec4f, previous: vec4f }
@group(0) @binding(0) var<uniform> u: Params;
const N: u32 = 25u;
fn hash(n: f32) -> f32 { return fract(sin(n * 127.1 + 31.7) * 43758.5453); }
fn rotate(p: vec3f, angle: f32) -> vec3f {
  let c = cos(angle); let s = sin(angle);
  return vec3f(c*p.x+s*p.z,p.y,-s*p.x+c*p.z);
}
struct Follicle {position:vec4f,normal:vec4f,parameters:vec4f,variation:vec4f,triangle:array<vec4f,3>,barycentric:vec4f,rest:array<vec4f,25>}
@group(0) @binding(6) var<storage,read> follicles:array<Follicle>;
@group(0) @binding(7) var<storage,read> collisionRadii:array<f32>;
fn root(id:u32)->vec3f {return follicles[id].position.xyz;}
fn rootAngle(id:u32)->f32 {return u.clock.z*follicles[id].position.w;}
fn surfaceWorld(id:u32)->vec3f {let f=follicles[id];var p=vec3f(0.0);for(var i=0u;i<3u;i++){p+=rotate(f.triangle[i].xyz,u.clock.z*f.triangle[i].w)*f.barycentric[i];}return p+rotate(f.normal.xyz*0.002,rootAngle(id));}
fn frameNormal(tangent:vec3f,hint:vec3f)->vec3f {
  let projected=hint-tangent*dot(hint,tangent);
  if(dot(projected,projected)>0.000001){return normalize(projected);}
  let axis=select(vec3f(0.0,1.0,0.0),vec3f(1.0,0.0,0.0),abs(tangent.y)>0.9);
  return normalize(cross(tangent,axis));
}
fn rest(id:u32,node:u32)->vec3f {return follicles[id].rest[node].xyz;}
fn shapeWorld(id:u32,node:u32)->vec3f {var p=rest(id,node);if(node>2u){p=collide(p);}return surfaceWorld(id)+rotate(p-root(id),rootAngle(id));}
fn renderSegments()->u32 {
  if(u.detail.z>0.5){return u32(u.detail.z); }
  if(u.settings.w>2.5 && u.settings.w<4.5){return 96u;}
  if(u.settings.w>1.5){return 80u;}
  if(u.settings.w>0.5){return 64u;}
  return 48u;
}

fn collide(point: vec3f) -> vec3f {
  var p = point;
  let center=vec3f(0.0,1.1,0.0);let q=p-center;let d=length(q);
  if(p.y>0.28 && d>0.00001){let direction=q/d;let uv=vec2f(atan2(direction.z,direction.x)/6.283185+0.5,acos(clamp(direction.y,-1.0,1.0))/3.141593);
   let xy=vec2i(floor(uv*vec2f(128.0,64.0)-0.5));var radius=0.0;
   for(var y=0;y<2;y++){for(var x=0;x<2;x++){let c=vec2i((xy.x+x+128)%128,clamp(xy.y+y,0,63));radius=max(radius,collisionRadii[u32(c.x+c.y*128)]);}}
   if(d<radius+0.003){p=center+direction*(radius+0.003);}
  }
  // Neck and a broad shoulder ellipsoid keep the hanging tips outside the bust.
  let shoulderCenter = vec3f(0.0,-0.46,0.0);
  let shoulderRadii = vec3f(1.78,0.32,0.66);
  let shoulderQ = (p-shoulderCenter)/shoulderRadii;
  let sd = length(shoulderQ);
  if (sd<1.0) { p = shoulderCenter+shoulderQ/max(sd,0.0001)*shoulderRadii; }
  let neckQ = (p-vec3f(0.0,0.3,0.0))/vec3f(0.32,0.47,0.33);
  let nd = length(neckQ);
  if (nd<1.0) { p=vec3f(0.0,0.3,0.0)+neckQ/max(nd,0.0001)*vec3f(0.32,0.47,0.33); }
  p.y=max(p.y,-2.5);
  return p;
}

fn keyDirection(p:vec3f)->vec3f {
 if(u.celestial.z>0.0){return sunDirection();}
 if(u.look.y>0.0){return -sunDirection();}
 var position=u.studio_light.xyz;
 if(u.viewport.w>0.5){position=vec3f(-3.0,3.0,1.5);}
 if(u.viewport.w>1.5 && u.viewport.w<2.5){position=vec3f(0.0,2.0,-3.5);}
 let delta=position-p;return delta/max(length(delta),0.0001);
}
fn keyColor()->vec3f {
 if(u.celestial.z>0.0){return mix(vec3f(1.0,0.45,0.18),vec3f(1.0,0.98,0.9),smoothstep(0.0,0.5,sunDirection().y))*u.celestial.z;}
 if(u.look.y>0.0){return vec3f(0.6,0.75,1.0)*u.look.y;}
 if(u.viewport.w>2.5){return vec3f(1.35,0.95,0.65)*u.look.w;}
 return vec3f(u.look.w);
}
@group(1) @binding(0) var environmentMap:texture_2d<f32>;
@group(1) @binding(1) var environmentSampler:sampler;
fn sunDirection()->vec3f {
 return normalize(vec3f(sin(u.celestial.x)*cos(u.celestial.y),sin(u.celestial.y),cos(u.celestial.x)*cos(u.celestial.y)));
}
fn proceduralSky(direction:vec3f)->vec3f {
 let daylight=smoothstep(-0.18,0.15,sunDirection().y);
 let horizon=pow(1.0-max(direction.y,0.0),3.0);
 let day=mix(vec3f(0.12,0.3,0.65),vec3f(0.65,0.7,0.8),horizon);
 let dusk=vec3f(0.8,0.18,0.045)*pow(max(dot(direction,sunDirection()),0.0),8.0)*(1.0-smoothstep(0.0,0.25,abs(sunDirection().y)));
 let cell=floor(rotate(direction,u.env.y)*900.0);
 let star=select(0.0,1.0,hash(dot(cell,vec3f(13.0,37.0,73.0)))>0.9985)*(1.0-daylight)*max(direction.y,0.0);
 return mix(vec3f(0.001,0.002,0.006),day,daylight)+dusk+vec3f(star)*u.illumination.w;
}
fn environmentRadiance(direction:vec3f,roughness:f32,background:bool)->vec3f {
 let d=normalize(rotate(direction,u.env.y));
 var color=proceduralSky(direction);
 if((u.env.x>0.5 && u.env.x<2.5)||u.env.x>3.5){
   let uv=vec2f(atan2(d.z,d.x)/6.283185+0.5,acos(clamp(d.y,-1.0,1.0))/3.141593);
   let maximum=f32(textureNumLevels(environmentMap)-1u);
   let level=select(max(u.illumination.y,roughness*maximum),u.illumination.x,background);
   color=textureSampleLevel(environmentMap,environmentSampler,uv,clamp(level,0.0,maximum)).rgb;
 }
 if(u.env.x<0.5){return vec3f(0.025,0.039,0.039);}
 let sun=sunDirection();
 if(background){
   color+=vec3f(15.0,12.0,8.0)*u.celestial.z*smoothstep(cos(u.look.x*1.2),cos(u.look.x*0.8),dot(direction,sun));
   let moon=-sun;
   let disc=smoothstep(cos(0.014),cos(0.010),dot(direction,moon));
   let moonX=dot(direction,frameNormal(moon,vec3f(1.0,0.0,0.0)));
   let phase=smoothstep(-0.003,0.003,moonX+(u.celestial.w*2.0-1.0)*0.014);
   color+=vec3f(1.1,1.3,1.7)*u.look.y*disc*phase;
 }
 return color*exp2(u.env.z);
}
fn environmentDiffuse(normal:vec3f)->vec3f {
 if(u.env.x<0.5){return vec3f(0.0);}
 if(u.env.x>2.5 && u.env.x<3.5){return environmentRadiance(normal,1.0,false)*0.5;}
 let n=normalize(rotate(normal,u.env.y));let x=n.x;let y=n.y;let z=n.z;
 let b=array<f32,9>(0.282095,0.488603*y,0.488603*z,0.488603*x,1.092548*x*y,1.092548*y*z,0.315392*(3.0*z*z-1.0),1.092548*x*z,0.546274*(x*x-y*y));
 var color=vec3f(0.0);
 for(var i=0u;i<9u;i++){color+=u.irradiance[i].rgb*b[i];}
 return max(color,vec3f(0.0))*exp2(u.env.z);
}

fn hairPigment(random:f32,chosenColor:f32)->vec4f {
 let pigments=array<vec2f,14>(vec2f(2.8000,0.0400),vec2f(0.6500,0.1800),vec2f(0.1000,0.0700),vec2f(0.0040,0.0010),vec2f(6.0000,0.0200),vec2f(0.2300,0.0450),vec2f(0.3200,0.7000),vec2f(0.0120,0.0080),vec2f(0.0600,1.0500),vec2f(1.1000,0.0800),vec2f(0.0350,0.2400),vec2f(3.0000,0.0200),vec2f(1.7000,0.1400),vec2f(0.8000,0.0180));
 let color=min(13u,u32(chosenColor));var c=pigments[color];
 if(color==11u && random<0.60){c=vec2f(0.004,0.001);}
 let absorption=(c.x*vec3f(0.419,0.697,1.37)+c.y*vec3f(0.187,0.4,1.05))*mix(0.92,1.08,random);
 return vec4f(absorption,u.material.x*mix(0.94,1.06,random));
}
fn pigmentVariation(group:f32,t:f32)->f32 {return mix(0.96,1.04,hash(group+22.0))*mix(1.08,0.94,smoothstep(0.02,1.0,t));}


@group(0) @binding(1) var<storage,read_write> nodes: array<Node>;

fn clumpId(id:u32)->u32 {
 let r=root(id);
 return min(2u,u32(clamp((2.1-r.y)*1.5,0.0,2.0)))*64u+min(63u,u32((atan2(r.z,r.x)+3.141593)/6.283186*64.0));
}
struct Clump { x:atomic<i32>, y:atomic<i32>, z:atomic<i32>, vx:atomic<i32>, vy:atomic<i32>, vz:atomic<i32>, count:atomic<u32> }
@group(0) @binding(4) var<storage,read_write> bonds:array<f32>;
@group(0) @binding(3) var<storage,read_write> clumps:array<Clump>;
struct Cell { count: atomic<u32>, vx: atomic<i32>, vy: atomic<i32>, vz: atomic<i32>, rest:atomic<u32>, optical:atomic<u32>, key:atomic<u32>, rim:atomic<u32>, studio:atomic<u32> }
@group(0) @binding(2) var<storage,read_write> grid: array<Cell>;
fn cellPosition(p:vec3f)->vec3i { return clamp(vec3i(floor((p+vec3f(4.0,3.0,4.0))*4.0)),vec3i(0),vec3i(31)); }
fn cellIndex(p:vec3i)->u32 {let c=vec3u(clamp(p,vec3i(0),vec3i(31)));return c.x+c.y*32u+c.z*1024u;}
fn density(p:vec3i)->f32{return f32(atomicLoad(&grid[cellIndex(p)].count));}
@compute @workgroup_size(128)
fn clearRest(@builtin(global_invocation_id) id:vec3u){if(id.x<32768u){atomicStore(&grid[id.x].rest,0u);}}
@compute @workgroup_size(128)
fn seedRest(@builtin(global_invocation_id) id:vec3u){if(id.x<u32(u.counts.x)*N && follicles[id.x/N].parameters.x>0.001){atomicAdd(&grid[cellIndex(cellPosition(rotate(nodes[id.x].position.xyz,-u.clock.z)))].rest,1u);}}
@compute @workgroup_size(128)
fn clearGrid(@builtin(global_invocation_id) id:vec3u){if(id.x>=32768u){return;}atomicStore(&grid[id.x].count,0u);atomicStore(&grid[id.x].vx,0);atomicStore(&grid[id.x].vy,0);atomicStore(&grid[id.x].vz,0);atomicStore(&grid[id.x].optical,0u);
if(id.x<4800u){atomicStore(&clumps[id.x].x,0);atomicStore(&clumps[id.x].y,0);atomicStore(&clumps[id.x].z,0);atomicStore(&clumps[id.x].vx,0);atomicStore(&clumps[id.x].vy,0);atomicStore(&clumps[id.x].vz,0);atomicStore(&clumps[id.x].count,0u);}}
@compute @workgroup_size(128)
fn scatterGrid(@builtin(global_invocation_id) id:vec3u){
 if(id.x>=u32(u.counts.x)*N){return;}
 if(follicles[id.x/N].parameters.x<=0.001){return;}
 let node=nodes[id.x];let index=cellIndex(cellPosition(rotate(node.position.xyz,-u.clock.z)));
 let velocity=vec3i(clamp((node.position.xyz-node.previous.xyz)/u.clock.y,vec3f(-20.0),vec3f(20.0))*256.0);

 let j=id.x%N;let group=clumpId(id.x/N)*N+j;
 let fixed=vec3i(clamp(node.position.xyz,vec3f(-8.0),vec3f(8.0))*4096.0);
 atomicAdd(&clumps[group].x,fixed.x);atomicAdd(&clumps[group].y,fixed.y);atomicAdd(&clumps[group].z,fixed.z);
 atomicAdd(&clumps[group].vx,velocity.x);atomicAdd(&clumps[group].vy,velocity.y);atomicAdd(&clumps[group].vz,velocity.z);atomicAdd(&clumps[group].count,1u);
 if(j>0u){
  let previous=nodes[id.x-1u].position.xyz;
  let weight=u32(clamp(length(node.position.xyz-previous)*100000.0/u.counts.x*64.0,0.0,65535.0));
  for(var k=0u;k<4u;k++){let sample=mix(previous,node.position.xyz,(f32(k)+0.5)/4.0);atomicAdd(&grid[cellIndex(cellPosition(rotate(sample,-u.clock.z)))].optical,weight);}
 }
 atomicAdd(&grid[index].count,1u);atomicAdd(&grid[index].vx,velocity.x);atomicAdd(&grid[index].vy,velocity.y);atomicAdd(&grid[index].vz,velocity.z);
}

@compute @workgroup_size(64)
fn initialize(@builtin(global_invocation_id) invocation: vec3u) {
  let id=invocation.x;
  if (id>=u32(u.counts.x)) { return; }
  for (var j=0u;j<N;j++) {
    let p=shapeWorld(id,j);
    bonds[id*N+j]=1.0;
    nodes[id*N+j]=Node(vec4f(p,select(0.0,follicles[id].parameters.z,j==1u)),vec4f(p,select(length(p-shapeWorld(id,select(j-1u,0u,j==0u))),u.clock.z,j==0u)));
  }
}

// The adjacent length constraints form a tridiagonal system. Solve their
// corrections together so that movement is shared across the free nodes.
fn solveStretch(points: ptr<function, array<vec3f,25>>, lengths: ptr<function, array<f32,25>>) {
  var direction: array<vec3f,25>;
  var diagonal: array<f32,25>;
  var coupling: array<f32,25>;
  var correction: array<f32,25>;
  for (var j = 3u; j < N; j++) {
    let delta = (*points)[j] - (*points)[j-1u];
    let segment = length(delta);
    direction[j] = select(vec3f(0.0,-1.0,0.0), delta / max(segment,0.000001), segment > 0.000001);
    correction[j] = (*lengths)[j] - segment;
    diagonal[j] = 1.0;
    if (j > 3u) {
      let previous = j-1u;
      coupling[j] = -dot(direction[j], direction[previous]);
      let factor = coupling[j] / diagonal[previous];
      diagonal[j] = 2.0 - factor * coupling[j];
      correction[j] -= factor * correction[previous];
    }
  }
  for (var j = N-1u; j >= 3u; j--) {
    if (j+1u < N) { correction[j] -= coupling[j+1u] * correction[j+1u]; }
    correction[j] /= diagonal[j];
  }
  for (var j = 3u; j < N; j++) {
    var offset = direction[j] * correction[j];
    if (j+1u < N) { offset -= direction[j+1u] * correction[j+1u]; }
    (*points)[j] += offset * 0.85;
  }
}

fn projectContacts(points: ptr<function, array<vec3f,25>>, contacts: ptr<function, array<vec3f,25>>, angle: f32) {
  for (var j = 3u; j < N; j++) {
    let before = (*points)[j];
    let local = rotate(before, -angle);
    let offset = collide(local) - local;
    if (dot(offset, offset) > 0.000000000001) {
      let worldOffset = rotate(offset, angle);
      (*points)[j] += worldOffset;
      (*contacts)[j] = normalize(worldOffset);
    }
  }
}

@compute @workgroup_size(64)
fn updateRoots(@builtin(global_invocation_id) invocation:vec3u){let id=invocation.x;if(id>=u32(u.counts.x)){return;}
 if(abs(nodes[id*N+1u].position.w-follicles[id].parameters.z)<0.1){return;}
 for(var j=0u;j<N;j++){let p=shapeWorld(id,j);bonds[id*N+j]=1.0;nodes[id*N+j]=Node(vec4f(p,select(0.0,follicles[id].parameters.z,j==1u)),vec4f(p,select(length(p-shapeWorld(id,select(j-1u,0u,j==0u))),u.clock.z,j==0u)));}}
@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) invocation: vec3u) {
  let id=invocation.x;
  if (id>=u32(u.counts.x)) { return; }

  var p: array<vec3f,25>;
  var old: array<vec3f,25>;
  var shape: array<vec3f,25>;
  var contact: array<vec3f,25>;
  var lengths: array<f32,25>;
  var twistLengths:array<f32,25>;
  let dt=u.clock.y;
  let angle=u.clock.z;
  let base=id*N;
  if(follicles[id].parameters.x<=0.001){return;}
  let movingRoot = abs(angle - nodes[base].previous.w) > 0.00001;
  let newGrab=abs(nodes[base+2u].position.w-u.anchor.w)>0.1;
  var weights:array<f32,25>;
  var predictedEnergy = 0.0;
  for (var j=0u;j<N;j++) {
    old[j]=nodes[base+j].position.xyz;
    shape[j]=shapeWorld(id,j);
    contact[j]=vec3f(0.0);
    weights[j]=0.0;
    if(u.physics.z>0.5 && j>2u){weights[j]=nodes[base+j].position.w;if(newGrab){weights[j]=1.0-smoothstep(0.0,u.grab.w,distance(old[j],u.anchor.xyz));}}
    lengths[j]=0.0;
    if (j>0u) { lengths[j]=length(shape[j]-shape[j-1u]); }
    if(j>2u){twistLengths[j]=length(shape[j]-shape[j-3u]);}
    if (j < 3u) {
      p[j] = shape[j];
      continue;
    }
    let velocity=(old[j]-nodes[base+j].previous.xyz)/dt;
    var airflow=vec3f(0.0);
    for(var sourceIndex=0u;sourceIndex<min(u32(u.sourceControl.x),16u);sourceIndex++){
      let source=u.sources[sourceIndex];
      let delta=old[j]-source.position.xyz;
      let radius=max(length(delta),0.001);
      let alignment=dot(delta/radius,source.direction.xyz);
      let cone=smoothstep(source.position.w,min(1.0,source.position.w+0.12),alignment);
      let intensity=source.direction.w*cone/(1.0+0.06*radius*radius);
      let phase=source.variation.x*0.013+source.variation.y*0.071;
      let localTime=source.variation.w;
      let noise=vec3f(sin(localTime*1.7+dot(old[j],vec3f(2.1,0.7,1.3))+phase),sin(localTime*1.3+old[j].z*2.0+phase*0.7),cos(localTime*1.9+old[j].x*2.2+phase*1.3));
      let transverse=noise-source.direction.xyz*dot(noise,source.direction.xyz);
      airflow+=(source.direction.xyz+transverse*source.variation.z*0.3)*intensity;
    }
    let speed=length(airflow);
    let air=airflow/max(speed,0.000001)*(18.0*tanh(speed/18.0));
    let next=nodes[base+min(j+1u,N-1u)].position.xyz;
    let previous=nodes[base+select(j-1u,0u,j==0u)].position.xyz;
    let tangent=normalize(next-previous+vec3f(0.00001));
    let relative=air-velocity;
    // Most drag acts across the fiber; a smaller component acts along its axis.
    let drag=(relative-tangent*dot(relative,tangent)*0.8)*2.5;
    var interaction=vec3f(0.0);
    if(u.physics.x>0.0 || u.physics.y>0.0){
      let cell=cellPosition(rotate(old[j],-angle));let index=cellIndex(cell);let count=max(density(cell),1.0);
      let gradient=vec3f(density(cell+vec3i(1,0,0))-density(cell-vec3i(1,0,0)),density(cell+vec3i(0,1,0))-density(cell-vec3i(0,1,0)),density(cell+vec3i(0,0,1))-density(cell-vec3i(0,0,1)));
      let average=vec3f(f32(atomicLoad(&grid[index].vx)),f32(atomicLoad(&grid[index].vy)),f32(atomicLoad(&grid[index].vz)))/(256.0*count);
      let restCount=f32(atomicLoad(&grid[index].rest));
      let packing=max(0.0,count/max(restCount*1.2,u.counts.x*0.025)-1.0);
      interaction=rotate(-gradient/max(length(gradient),1.0),angle)*min(packing,2.0)*u.physics.x*1.5;
      interaction+=(average-velocity)*u.physics.y*min(count/8.0,1.0)*4.0;
    }

    let group=clumpId(id)*N+j;let members=f32(atomicLoad(&clumps[group].count));
    if(u.graphics.y>0.0 && members>1.0){
      let center=vec3f(f32(atomicLoad(&clumps[group].x)),f32(atomicLoad(&clumps[group].y)),f32(atomicLoad(&clumps[group].z)))/(4096.0*members);
      let average=vec3f(f32(atomicLoad(&clumps[group].vx)),f32(atomicLoad(&clumps[group].vy)),f32(atomicLoad(&clumps[group].vz)))/(256.0*members);
      let gap=center-old[j];let lateral=gap-tangent*dot(gap,tangent);let separation=length(lateral);

      let detach=max(smoothstep(0.2*u.detail.y,0.38*u.detail.y,separation),smoothstep(2.0,5.0,length(average-velocity)));
      let attach=(1.0-smoothstep(0.08*u.detail.y,0.14*u.detail.y,separation))*(1.0-smoothstep(0.5,1.5,length(average-velocity)));
      let rate=attach*1.5+detach*8.0;
      let equilibrium=attach*1.5/max(rate,0.0001);
      let bond=mix(bonds[base+j],equilibrium,1.0-exp(-rate*dt));
      bonds[base+j]=bond;
      let force=lateral*max(0.0,separation-0.045*u.detail.y)/max(separation,0.001)*9.0+(average-velocity)*0.65;
      interaction+=force*min(1.0,2.0/max(length(force),0.001))*u.graphics.y*follicles[id].variation.w*bond*(1.0-pow(f32(j)/24.0,2.0)*0.85);
    }
    let acceleration=vec3f(0.0,-9.81,0.0)+drag+interaction;
    var displacement=velocity*dt*exp(-1.2*dt)+acceleration*dt*dt;
    displacement*=min(1.0,0.08/max(length(displacement),0.000001));
    p[j]=old[j]+displacement;
    predictedEnergy += dot(displacement, displacement);
  }

  // Root attachments fix the initial direction and curvature at the scalp.
  // Distance bending preserves curvature without pulling the entire groom home.
  for (var iteration=0u;iteration<u32(u.physics.w);iteration++) {
    p[0]=shape[0]; p[1]=shape[1]; p[2]=shape[2];
    for (var k=3u;k<N;k++) {
      let j=select(k,N+2u-k,iteration%2u==1u);
      let a=j-1u;
      let delta=p[j]-p[a];
      let correction=delta*(1.0-lengths[j]/max(length(delta),0.000001));
      let parentWeight=select(0.5,0.0,a<3u);
      p[j]-=correction*(1.0-parentWeight);
      p[a]+=correction*parentWeight;
      let bend=p[j]-p[j-2u];
      let restBend=length(shape[j]-shape[j-2u]);
      let bendCorrection=bend*(1.0-restBend/max(length(bend),0.000001))*u.settings.z*select(0.4,0.8,u.settings.w>1.5)*mix(1.0,0.6,f32(j)/24.0);
      let bendWeight=select(0.5,0.0,j<5u);
      p[j]-=bendCorrection*(1.0-bendWeight);
      if (j>4u) { p[j-2u]+=bendCorrection*bendWeight; }
      // Three-hop distances preserve local helical torsion alongside bending.
      // Every correction acts on its two endpoints in the current configuration.
      if(j>=3u){
        let delta3=p[j]-p[j-3u];
        let correction3=delta3*(1.0-twistLengths[j]/max(length(delta3),0.000001))*u.settings.z*select(0.35,0.65,u.settings.w>1.5)*mix(1.0,0.6,f32(j)/24.0);
        let parentWeight=select(0.5,0.0,j<6u);
        p[j]-=correction3*(1.0-parentWeight);
        if(j>=6u){p[j-3u]+=correction3*parentWeight;}
      }
      if(u.physics.z>0.5){p[j]=mix(p[j],u.grab.xyz,weights[j]*0.35);}
    }

    projectContacts(&p, &contact, angle);
  }
  for (var iteration = 0u; iteration < 8u; iteration++) {
    solveStretch(&p, &lengths);
    projectContacts(&p, &contact, angle);
  }
  var displacement: array<vec3f,25>;
  for (var j = 0u; j < N; j++) { displacement[j] = p[j] - old[j]; }
  for (var sweep = 0u; sweep < 2u; sweep++) {
    for (var k = 3u; k < N; k++) {
      let j = select(k, N+2u-k, sweep == 1u);
      let parent = j-1u;
      let delta = p[j] - p[parent];
      let axis = delta / max(length(delta), 0.000001);
      let normalMotion = axis * dot(displacement[j]-displacement[parent], axis);
      let parentWeight = select(0.5, 0.0, parent < 3u);
      displacement[j] -= normalMotion * (1.0-parentWeight);
      displacement[parent] += normalMotion * parentWeight;
    }
  }
  var solvedEnergy = 0.0;
  for (var j = 3u; j < N; j++) {
    if (dot(contact[j],contact[j]) > 0.0) {
      displacement[j] = (displacement[j] - contact[j]*dot(displacement[j],contact[j])) * 0.8;
    }
    displacement[j] *= min(1.0, 0.04 / max(length(displacement[j]),0.000001));
    solvedEnergy += dot(displacement[j],displacement[j]);
  }
  // Projection should not supply energy. A moving scalp or grab can do work.
  var damping = exp(-2.5 * dt);
  if (!movingRoot && u.physics.z < 0.5 && solvedEnergy > predictedEnergy) {
    damping *= sqrt(predictedEnergy / solvedEnergy);
  }
  for (var j = 0u; j < N; j++) {
    let movement = displacement[j] * select(damping,1.0,j<3u);
    nodes[base+j] = Node(vec4f(p[j],weights[j]), vec4f(p[j]-movement,select(lengths[j],angle,j==0u)));
  }
  nodes[base].position.w=0.0;
  nodes[base+1u].position.w=follicles[id].parameters.z;
  nodes[base+2u].position.w=u.anchor.w;

}
fn opticalAt(world:vec3f)->f32 {
 let q=(rotate(world,-u.clock.z)+vec3f(4.0,3.0,4.0))*4.0-0.5;
 if(any(q<vec3f(0.0))||any(q>vec3f(30.0))){return 0.0;}
 let cell=vec3u(floor(q));let f=fract(q);var value=0.0;
 for(var z=0u;z<2u;z++){for(var y=0u;y<2u;y++){for(var x=0u;x<2u;x++){
  let c=cell+vec3u(x,y,z);let w=mix(1.0-f,f,vec3f(f32(x),f32(y),f32(z)));
  value+=f32(atomicLoad(&grid[c.x+c.y*32u+c.z*1024u].optical))*w.x*w.y*w.z/256.0;
 }}}
 return value;
}
fn integrateShadow(p:vec3f,L:vec3f)->f32 {
 if(u.graphics.x<0.5){return 1.0;}
 let samples=select(select(4u,8u,u.graphics.x>1.5),16u,u.graphics.x>2.5);
 let step=2.0/f32(samples);var tau=0.0;
 for(var i=0u;i<samples;i++){tau+=opticalAt(p+L*(0.035+(f32(i)+0.5)*step))*step;}
 return exp(-tau*u.graphics.w*0.018*u.detail.x);
}

// Cache light attenuation in a separate dispatch to amortize it over followers.
@compute @workgroup_size(128)
fn shadeGrid(@builtin(global_invocation_id) id:vec3u){
 if(id.x>=32768u){return;}
 let c=vec3f(f32(id.x%32u),f32((id.x/32u)%32u),f32(id.x/1024u));
 let world=rotate((c+0.5)/4.0-vec3f(4.0,3.0,4.0),u.clock.z);
 let key=integrateShadow(world,keyDirection(world));
 var rim=1.0;
 if(u.look.w>0.0){rim=integrateShadow(world,normalize(vec3f(2.0,2.8,-3.0)-world));}
 atomicStore(&grid[id.x].key,u32(key*65535.0));

 let delta=u.studio_light.xyz-world;
 let studioDirection=delta/max(length(delta),0.0001);
 var studio=key;
 if((u.look.w>0.0 || dot(studioDirection,keyDirection(world))>0.7) && dot(studioDirection,keyDirection(world))<0.999){studio=integrateShadow(world,studioDirection);}
 atomicStore(&grid[id.x].studio,u32(studio*65535.0));
 atomicStore(&grid[id.x].rim,u32(rim*65535.0));
}
