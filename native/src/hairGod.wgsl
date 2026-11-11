
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


@group(0) @binding(1) var<storage,read> nodes: array<Node>;

struct OpticalCell { count:u32, vx:i32, vy:i32, vz:i32, rest:u32, optical:u32, key:u32, rim:u32, studio:u32 }
@group(0) @binding(2) var<storage,read> opticalGrid:array<OpticalCell>;

fn transmittance(p:vec3f,L:vec3f)->f32 {
 if(u.graphics.x<0.5){return 1.0;}
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
struct HairVertex {
 @builtin(position) clip:vec4f,
 @location(0) world:vec3f,
 @location(1) tangent:vec3f,
 @location(2) attributes:vec4f,
 @location(3) edge:f32,
 @location(4) @interpolate(flat) optical:vec4f,
 @location(5) @interpolate(flat) local:vec4f,
}
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
@group(0) @binding(5) var<storage,read> cachedCurves:array<CachedCurve>;
fn fiberSample(fiber:u32,t:f32)->CurveSample {
  let guide=fiber%u32(u.counts.x);
  let cached=cachedCurves[guide*97u+u32(round(t*f32(renderSegments())))];
  var curve=CurveSample(cached.point.xyz,cached.tangent.xyz);
  if(u.detail.w>0.5){curve=guideSample(guide,t);}
  let center=curve.point;
  var X:vec3f;var Y:vec3f;var startPhase=0.0;
  if((u32(u.look.z)&1u)!=0u){let frame=cachedCurves[8192u*97u+guide*97u+u32(round(t*f32(renderSegments())))];X=frame.point.xyz;Y=frame.tangent.xyz;startPhase=frame.point.w;}else{
  let T=normalize(curve.tangent+vec3f(0.000001));
  let reference=rotate(follicles[guide].normal.xyz,rootAngle(guide));
  X=frameNormal(T,reference);Y=cross(T,X);
  }
  let seed=hash(f32(fiber)+7.0);
  let phase=seed*6.283185;
  let localFly=select(u.material.w,follicles[guide].variation.y,follicles[guide].variation.y>=0.0);
  let fly=select(0.0,1.0,hash(f32(fiber)+37.0)>1.0-localFly);
  let rootSpacing=min(0.075,0.035*sqrt(4096.0/u.counts.x));
  let spread=min(1.0,follicles[guide].parameters.w/0.4)*mix(rootSpacing,0.025*u.detail.y,t)*mix(1.0,2.2,fly)*sqrt(hash(f32(fiber)+93.0));
  var offset=(X*cos(phase)+Y*sin(phase))*spread*(0.8+0.2*t);
  var derivative=curve.tangent;
  let coils=u.settings.w>2.5 && u.settings.w<4.5;
  if(coils){
    let radius=follicles[guide].parameters.y*min(1.0,follicles[guide].parameters.w/1.8)*mix(0.012,0.025,hash(f32(fiber/8u)+11.0));
    let turns=mix(5.0,8.0,hash(f32(fiber/8u)+13.0));
    let angle=phase+t*turns*6.283185;
    offset+=(X*(sin(angle)-sin(phase))+Y*(cos(angle)-cos(phase)))*radius*smoothstep(0.0,0.1,t);
    derivative+=(X*cos(angle)-Y*sin(angle))*radius*turns*6.283185*smoothstep(0.0,0.1,t);
  }

  if(u.settings.w>0.5 && !coils){
    // Correlated detail follows the simulated curve's current local frame.
    var start=startPhase;
    if((u32(u.look.z)&1u)==0u){
    let r=root(guide);
    let lock=floor((atan2(r.z,r.x)+3.141593)/6.283186*12.0)+floor(f32(guide)/u.counts.x*2.0)*12.0;
    start=hash(lock+131.0)*6.283185;
    }
    let turns=select(1.8,3.0,u.settings.w>1.5);
    let radius=select(0.04,0.045,u.settings.w>1.5)*follicles[guide].parameters.y*min(1.0,follicles[guide].parameters.w/1.8);
    let angle=start+t*turns*6.283185;
    let fade=smoothstep(0.0,0.12,t);
    offset+=(X*(sin(angle)-sin(start))+Y*(cos(angle)-cos(start))*0.6)*radius*fade;
    derivative+=(X*cos(angle)-Y*sin(angle)*0.6)*radius*turns*6.283185*fade;
  }
  offset+=X*sin(t*4.0+phase)*fly*0.06*t;
  var position=center+offset;
  if(t<0.35){position=rotate(collide(rotate(position,-rootAngle(guide))),rootAngle(guide));}
  return CurveSample(position,normalize(derivative+vec3f(0.000001)));
}
@vertex fn vertex(@builtin(vertex_index) index:u32,@builtin(instance_index) instance:u32)->HairVertex {
  let fiber=select(u32(f32(instance)*u.counts.x/max(u.counts.y,1.0)),instance,u.counts.y>=u.counts.x);
  let guide=fiber%u32(u.counts.x);let density=follicles[guide].parameters.x;
  if(density<=0.001 || hash(f32(fiber)+237.0)>density){var empty:HairVertex;empty.clip=vec4f(2.0,2.0,2.0,1.0);return empty;}
  let segments=renderSegments();
  let t=f32(index/2u)/f32(segments);
  let side=select(-1.0,1.0,index%2u==1u);
  let curve=fiberSample(fiber,t);
  let p=curve.point;let T=curve.tangent;
  // Ribbon width follows the projected direction perpendicular to the tangent.
  let perpendicular=normalize(cross(T,normalize(u.eye.xyz-p))+vec3f(0.000001));
  var clip=u.matrix*vec4f(p,1.0);
  let adjacent=u.matrix*vec4f(p+perpendicular*0.001,1.0);
  let screen=normalize((adjacent.xy/adjacent.w-clip.xy/clip.w)*u.viewport.xy+vec2f(0.000001));
  let taper=1.0-smoothstep(0.85,1.0,t)*0.85;
  let projected=max(0.001,u.material.z*follicles[guide].variation.x*0.001*(0.85+hash(f32(fiber)+18.0)*0.3)*taper*u.viewport.y/max(clip.w,0.001));
  let represented=max(u.counts.z,200000.0)*u.detail.x/max(u.counts.y,1.0);
  let footprint=projected*sqrt(represented);
  let pixels=max(footprint,0.8);
  clip=vec4f(clip.xy+screen*side*pixels/u.viewport.xy*clip.w,clip.zw);
  let r=root(fiber%u32(u.counts.x));
  let group=min(2u,u32(clamp((2.1-r.y)*1.5,0.0,2.0)))*64u+min(63u,u32((atan2(r.z,r.x)+3.141593)/6.283186*64.0));
  var out:HairVertex;
  out.local=follicles[guide].variation;out.clip=clip;out.world=p;out.tangent=T;out.edge=side;
  out.attributes=vec4f(hash(f32(fiber)+1.0),t,min(1.0,projected*represented/pixels),f32(group));
  out.optical=vec4f(0.0);if((u32(u.look.z)&2u)!=0u){out.optical=hairPigment(out.attributes.x,select(u.counts.w,out.local.z,out.local.z>=0.0));}
  return out;
}

fn gaussian(x:f32,width:f32)->f32 {return exp(-0.5*x*x/(width*width))/(width*2.506628);}
fn scattering(T:vec3f,V:vec3f,L:vec3f,absorption:vec3f,roughness:f32)->vec3f {
 let ti=asin(clamp(dot(T,L),-0.999,0.999));let to=asin(clamp(dot(T,V),-0.999,0.999));let sum=ti+to;
 let vp=normalize(V-T*dot(T,V)+vec3f(0.000001));let lp=normalize(L-T*dot(T,L)+vec3f(0.000001));let azimuth=dot(vp,lp);
 let surface=normalize(vp+lp+vec3f(0.000001));
 let incidence=clamp(abs(dot(L,surface)),0.0,1.0);
 let F=0.0465+0.9535*pow(1.0-incidence,5.0);
 let transmission=exp(-absorption*(2.0/max(cos(ti/1.55),0.25)));
 let R=vec3f(F)*gaussian(sum+0.07,roughness)*mix(0.25,1.0,smoothstep(-0.8,0.8,azimuth))/3.926991;
 let TT=(1.0-F)*(1.0-F)*transmission*gaussian(sum-0.035,roughness*0.7)*pow(max(0.0,-azimuth),3.0)*0.75;
 let TRT=(1.0-F)*(1.0-F)*F*transmission*transmission*gaussian(sum-0.14,roughness*1.5)*mix(0.4,1.0,max(0.0,azimuth))/3.713274;
 if(u.material.y>0.5 && u.material.y<1.5){return R;}
 if(u.material.y>1.5 && u.material.y<2.5){return TT;}
 if(u.material.y>2.5 && u.material.y<3.5){return TRT;}
 return R+TT+TRT;
}
struct HairOutput { @location(0) color:vec4f, @builtin(sample_mask) mask:u32 }
@fragment fn fragment(v:HairVertex)->HairOutput {
 let coverage=clamp(v.attributes.z*(1.0-smoothstep(0.55,1.0,abs(v.edge))),0.0,1.0);
 // Stable stratification retains expected coverage below the MSAA threshold.

 let count=u32(clamp(floor(coverage*4.0+hash(v.attributes.x*65536.0+139.0)),0.0,4.0));
 let bits=(1u<<count)-1u;let rotation=u32(v.attributes.x*4.0);
 if((u32(u.look.z)&4u)!=0u && count==0u){discard;}

 let T=normalize(v.tangent);let V=normalize(u.eye.xyz-v.world);
 let normal=normalize(V-T*dot(V,T)+vec3f(0.000001));
 let diffuseEnv=environmentDiffuse(normal);
 // PBRT pigment concentration coefficients, measured per fiber diameter.
 var pigment=v.optical;
 if((u32(u.look.z)&2u)==0u){pigment=hairPigment(v.attributes.x,select(u.counts.w,v.local.z,v.local.z>=0.0));}
 let absorption=pigment.xyz*pigmentVariation(v.attributes.w,v.attributes.y);
 let roughness=pigment.w;
 let key=keyDirection(v.world);let rim=normalize(vec3f(2.0,2.8,-3.0)-v.world);
 let shadow=transmittance(v.world,key)*bodyVisibility(v.world,key);
 var lighting=scattering(T,V,key,absorption,roughness)*shadow*select(4.0,2.5,u.graphics.x>1.5)*keyColor();
 if(u.graphics.x>0.5){lighting+=scattering(T,V,rim,absorption,roughness)*transmittance(v.world,rim)*bodyVisibility(v.world,rim)*vec3f(1.5,1.65,1.9)*u.look.w;}
 if(u.graphics.x>1.5){
  let count=select(2u,6u,u.graphics.x>2.5);
  for(var i=0u;i<count;i++){let a=f32(i)*2.399963;let area=normalize(key+vec3f(cos(a),sin(a),0.0)*select(u.studio_light.w,u.look.x,u.celestial.z>0.0||u.look.y>0.0));lighting+=scattering(T,V,area,absorption,roughness)*shadow*1.5*keyColor()/f32(count);}
 }

 if((u.celestial.z>0.0||u.look.y>0.0)&&u.look.w>0.0){
   let delta=u.studio_light.xyz-v.world;let L=delta/max(length(delta),0.0001);
   lighting+=scattering(T,V,L,absorption,roughness)*4.0*u.look.w*transmittance(v.world,L)*bodyVisibility(v.world,L);
 }
 // Approximate diffuse transport from higher orders.
 let indirect=max(vec3f(0.015),keyColor()*0.35+diffuseEnv*u.illumination.z*0.5);
 lighting+=exp(-absorption*6.0)*(0.48+0.28*(1.0-shadow))*indirect;
 lighting+=vec3f(0.0465*0.12);


 if(u.env.x>0.5){
   lighting+=(exp(-absorption*1.6)*diffuseEnv*0.35
     +environmentRadiance(reflect(-V,normal),roughness,false)*0.0465
     +environmentRadiance(-V,roughness,false)*exp(-absorption*2.0)*0.08)*u.illumination.z;
 }
 if(u.material.y>3.5 && u.material.y<4.5){lighting=vec3f(shadow);}
 if(u.material.y>4.5){
  let guide=u32(v.attributes.w);
  lighting=0.15+0.7*vec3f(hash(f32(guide)+1.0),hash(f32(guide)+7.0),hash(f32(guide)+17.0));
 }
 return HairOutput(vec4f(clamp(lighting,vec3f(0.0),vec3f(65000.0)),1.0),((bits<<rotation)|(bits>>(4u-rotation)))&15u);
}
