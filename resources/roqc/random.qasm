OPENQASM 2.0;
include "qelib1.inc";
qreg q[8];
cx q[0],q[5];
rz(1.3553941036177939) q[1];
h q[2];
rz(0.6182132923511718) q[4];