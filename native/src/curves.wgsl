
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


@group(0) @binding(1) var<storage,read> nodes:array<Node>;
struct CurveSample { point:vec3f, tangent:vec3f }
fn guideSample(guide:u32,t:f32)->CurveSample {
  let sample=clamp(t,0.0,1.0)*24.0;
  let i=min(u32(sample),23u);
  let f=sample-f32(i);
  let a=nodes[guide*N+select(i-1u,0u,i==0u)].position.xyz;
  let b=nodes[guide*N+i].position.xyz;
  let c=nodes[guide*N+i+1u].position.xyz;
  let d=nodes[guide*N+min(i+2u,24u)].position.xyz;
  let m0=(c-a)*0.5;let m1=(d-b)*0.5;
  let f2=f*f;let f3=f2*f;
  let point=(2.0*f3-3.0*f2+1.0)*b+(f3-2.0*f2+f)*m0+(-2.0*f3+3.0*f2)*c+(f3-f2)*m1;
  let tangent=((6.0*f2-6.0*f)*b+(3.0*f2-4.0*f+1.0)*m0+(-6.0*f2+6.0*f)*c+(3.0*f2-2.0*f)*m1)*24.0;
  return CurveSample(point,tangent);
}

struct CachedCurve {point:vec4f,tangent:vec4f}
@group(0) @binding(5) var<storage,read_write> cachedCurves:array<CachedCurve>;
@compute @workgroup_size(128) fn prepareCurves(@builtin(global_invocation_id) id:vec3u){
 let guide=id.x/97u;let sample=id.x%97u;let segments=renderSegments();
 if(guide>=u32(u.counts.x)||sample>segments){return;}
 let curve=guideSample(guide,f32(sample)/f32(segments));
 cachedCurves[id.x]=CachedCurve(vec4f(curve.point,0.0),vec4f(curve.tangent,0.0));
 if((u32(u.look.z)&1u)!=0u){
  let T=normalize(curve.tangent+vec3f(0.000001));
  let r=root(guide);let reference=rotate(follicles[guide].normal.xyz,rootAngle(guide));
  let X=frameNormal(T,reference);let Y=cross(T,X);
  let lock=floor((atan2(r.z,r.x)+3.141593)/6.283186*12.0)+floor(f32(guide)/u.counts.x*2.0)*12.0;
  let start=hash(lock+131.0)*6.283185;
  cachedCurves[8192u*97u+id.x]=CachedCurve(vec4f(X,start),vec4f(Y,0.0));
 }
}
