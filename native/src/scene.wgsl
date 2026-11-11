
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



struct OpticalCell { count:u32, vx:i32, vy:i32, vz:i32, rest:u32, optical:u32, key:u32, rim:u32, studio:u32 }
@group(0) @binding(2) var<storage,read> opticalGrid:array<OpticalCell>;

fn transmittance(p:vec3f,L:vec3f)->f32 {
 if(u.graphics.x<0.5||u.clock.w>0.5){return 1.0;}
 let q=(rotate(p,-u.clock.z)+vec3f(4.0,3.0,4.0))*4.0-0.5;
 if(any(q<vec3f(0.0))||any(q>vec3f(30.0))){return 1.0;}
 let cell=vec3u(floor(q));let f=fract(q);var value=0.0;
 let useKey=dot(L,keyDirection(p))>0.7;
 let delta=u.studio_light.xyz-p;let useStudio=dot(L,delta/max(length(delta),0.0001))>0.7;
 for(var z=0u;z<2u;z++){for(var y=0u;y<2u;y++){for(var x=0u;x<2u;x++){
  let c=cell+vec3u(x,y,z);let w=mix(1.0-f,f,vec3f(f32(x),f32(y),f32(z)));
  let entry=opticalGrid[c.x+c.y*32u+c.z*1024u];
  value+=f32(select(select(entry.rim,entry.studio,useStudio),entry.key,useKey))*w.x*w.y*w.z/65535.0;
 }}}
 return value;
}
fn bodyVisibility(p:vec3f,L:vec3f)->f32 {
 let o=(rotate(p,-u.clock.z)-vec3f(0.0,1.1,0.0))/vec3f(0.624,0.865,0.574);
 let d=rotate(L,-u.clock.z)/vec3f(0.624,0.865,0.574);
 let b=dot(o,d);let a=dot(d,d);let disc=b*b-a*(dot(o,o)-1.0);
 if(disc>0.0 && (-b-sqrt(disc))/a>0.015){return 0.03;}return 1.0;
}

struct SceneVertex { @builtin(position) clip: vec4f, @location(0) normal: vec3f, @location(1) world: vec3f, @location(2) color: vec3f }
@vertex fn vertex(@location(0) position: vec3f,@location(1) normal: vec3f,@location(2) color: vec3f,@location(3) moving: f32) -> SceneVertex {
  var out: SceneVertex;
  let angle=u.clock.z*moving;
  let p=rotate(position,angle);
  out.clip=u.matrix*vec4f(p,1.0); out.normal=rotate(normal,angle); out.world=p; out.color=color;
  return out;
}
@fragment fn fragment(v: SceneVertex) -> @location(0) vec4f {
  let normal=normalize(v.normal);
  let light=keyDirection(v.world);
  let view=normalize(u.eye.xyz-v.world);
  let diffuse=max(dot(normal,light),0.0)*transmittance(v.world,light);
  let rim=pow(1.0-max(dot(normal,view),0.0),3.0);
  let ambientVisibility=sqrt(max(0.02,transmittance(v.world,normal)));
  var color=v.color*(vec3f(0.20)*ambientVisibility+diffuse*keyColor()*0.8)+vec3f(0.065,0.095,0.08)*rim;
  color+=pow(max(dot(normal,normalize(light+view)),0.0),60.0)*0.045;
  color+=(v.color*environmentDiffuse(normal)*u.illumination.z+environmentRadiance(reflect(-view,normal),0.6,false)*0.025*select(0.0,1.0,u.env.x>0.5))*ambientVisibility;

 if((u.celestial.z>0.0||u.look.y>0.0)&&u.look.w>0.0){
   let delta=u.studio_light.xyz-v.world;let L=delta/max(length(delta),0.0001);
   color+=v.color*max(dot(normal,L),0.0)*u.look.w*0.8*transmittance(v.world,L);
 }
 return vec4f(clamp(color*1.8,vec3f(0.0),vec3f(65000.0)),1.0);
}
